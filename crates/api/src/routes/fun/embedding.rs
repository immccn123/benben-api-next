use ndarray::{Array1, Array2};
use safetensors::SafeTensors;
use tokenizers::Tokenizer;

pub struct StaticEmbeddingModel {
    table: Array2<f32>,
    tokenizer: Tokenizer,
    dim: usize,
    vocab: usize,
}

impl StaticEmbeddingModel {
    pub fn load(
        model_path: &std::path::Path,
        tokenizer_path: &std::path::Path,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let data = std::fs::read(model_path)?;
        let tensors = SafeTensors::deserialize(&data)?;

        let names = tensors.names().to_vec();
        let tensor_name = names
            .iter()
            .find(|n| n.ends_with(".weight"))
            .ok_or_else(|| format!("No weight tensor found in {:?}", names))?;

        let tensor = tensors.tensor(tensor_name)?;
        let shape = tensor.shape();
        let vocab = shape[0];
        let dim = shape[1];
        let raw = tensor.data();
        let data: &[f32] =
            unsafe { std::slice::from_raw_parts(raw.as_ptr() as *const f32, raw.len() / 4) };

        let table = Array2::from_shape_vec((vocab, dim), data.to_vec())?;
        let tokenizer =
            Tokenizer::from_file(tokenizer_path).map_err(|e| format!("Tokenizer: {}", e))?;

        println!("[embedding] Loaded: vocab={vocab}, dim={dim}");
        Ok(Self {
            table,
            tokenizer,
            dim,
            vocab,
        })
    }

    pub fn dim(&self) -> usize {
        self.dim
    }

    pub fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error>> {
        texts.iter().map(|t| self.embed_one(t)).collect()
    }

    fn embed_one(&self, text: &str) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        let enc = self
            .tokenizer
            .encode(text, true)
            .map_err(|e| format!("Tokenize: {}", e))?;
        let ids = enc.get_ids();
        let mask = enc.get_attention_mask();

        if ids.is_empty() {
            return Ok(vec![0.0f32; self.dim]);
        }

        let mut sum = Array1::zeros(self.dim);
        let mut n: f32 = 0.0;
        for (&id, &m) in ids.iter().zip(mask.iter()) {
            if m == 0 || id as usize >= self.vocab {
                continue;
            }
            let row = self.table.row(id as usize);
            sum += &row;
            n += 1.0;
        }
        if n > 0.0 {
            sum /= n;
        }
        Ok(sum.to_vec())
    }
}

pub struct DuelState {
    pub model: StaticEmbeddingModel,
}

impl DuelState {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        use hf_hub::api::sync::Api;

        let model_id = "sentence-transformers/static-similarity-mrl-multilingual-v1";
        println!("[duel] Loading from HuggingFace: {model_id}");

        let api = Api::new()?;
        let repo = api.model(model_id.to_string());

        let model_path = repo.get("0_StaticEmbedding/model.safetensors")?;
        let tokenizer_path = repo.get("0_StaticEmbedding/tokenizer.json")?;

        let model = StaticEmbeddingModel::load(&model_path, &tokenizer_path)?;

        Ok(Self { model })
    }
}
