use std::collections::HashMap;

use ndarray::{Array2, Axis};
use rocket::State;
use rocket_db_pools::Connection;
use sea_orm::{entity::prelude::*, QueryOrder, QuerySelect};
use serde_json::json;

use super::embedding::DuelState;
use crate::db_pool::Db;
use crate::err::AppError;

use entity::feed as benben;
use entity::feed_takedown as benben_takedown;

lazy_static::lazy_static! {
    static ref JIEBA: jieba_rs::Jieba = jieba_rs::Jieba::new();
}

const STOP_WORDS: &[&str] = &[
    "一", "一个", "丁", "七", "万", "三", "上", "下", "不", "与", "且", "个", "中", "为", "主",
    "之", "也", "了", "于", "些", "人", "他", "们", "但", "你", "便", "信", "像", "儿", "其", "内",
    "再", "出", "到", "前", "又", "去", "及", "可", "各", "后", "吗", "吧", "听", "呢", "哎", "啊",
    "哦", "咦", "咧", "哩", "哪", "呃", "哼", "唯", "啥", "啦", "哟", "哇", "呀", "和", "在", "地",
    "她", "好", "如", "它", "对", "将", "就", "但", "已", "并", "得", "很", "怎", "所", "把", "拿",
    "是", "有", "来", "样", "此", "每", "比", "没", "然", "的", "看", "着", "被", "要", "说", "让",
    "给", "而", "能", "自", "若", "虽", "里", "那", "都", "还", "这", "进", "道", "问", "吗", "啊",
    "呀", "呢", "吧", "哈", "嘛", "呗", "最", "会", "从", "用", "向", "往", "朝", "按", "照", "以",
    "可", "能", "会", "要", "过", "起", "时", "后", "只", "个", "些", "点", "种", "次", "回", "遍",
    "很", "太", "真", "最", "更", "极", "挺", "满", "多", "少", "做", "作", "使", "叫", "受", "靠",
    "www", "com", "http", "https", "html", "htm",
];

#[get("/duel?<uid_a>&<uid_b>")]
pub async fn duel(
    conn: Connection<Db>,
    state: &State<DuelState>,
    uid_a: i32,
    uid_b: i32,
) -> Result<Json, AppError> {
    let db = &conn.into_inner();

    let (texts_a, count_a) = sample_and_preprocess(db, uid_a).await?;
    let (texts_b, count_b) = sample_and_preprocess(db, uid_b).await?;

    let embeddings_a = compute_embeddings(&state, &texts_a)
        .map_err(|e| AppError::ModelError(format!("Embedding failed: {}", e)))?;
    let embeddings_b = compute_embeddings(&state, &texts_b)
        .map_err(|e| AppError::ModelError(format!("Embedding failed: {}", e)))?;

    let topic_resonance = compute_topic_resonance(&embeddings_a, &embeddings_b);
    let divergence = compute_divergence(&embeddings_a, &embeddings_b);
    let entropy = compute_entropy(&embeddings_a, &embeddings_b);
    let duplicate_rate = compute_duplicate_rate(&texts_a, &texts_b);

    Ok(json!({
        "code": 200,
        "data": {
            "uid_a": uid_a,
            "uid_b": uid_b,
            "sample_count_a": count_a,
            "sample_count_b": count_b,
            "topic_resonance": topic_resonance,
            "divergence": divergence,
            "entropy": entropy,
            "duplicate_rate": duplicate_rate,
        }
    }))
}

export_routes!(duel);

// ==================== Data Sampling ====================

async fn sample_and_preprocess(
    db: &sea_orm::DatabaseConnection,
    uid: i32,
) -> Result<(Vec<String>, u64), AppError> {
    let total: u64 = benben::Entity::find()
        .left_join(benben_takedown::Entity)
        .filter(benben_takedown::Column::FeedId.is_null())
        .filter(benben::Column::UserId.eq(uid))
        .count(db)
        .await?;

    if total < 128 {
        return Err(AppError::InvalidParam(format!(
            "Insufficient data: uid = {uid}, total = {total}, need >= 128"
        )));
    }

    let limit = if total <= 512 {
        total
    } else {
        let one_year_ago = chrono::Utc::now() - chrono::Duration::days(365);
        let one_year_count: u64 = benben::Entity::find()
            .left_join(benben_takedown::Entity)
            .filter(benben_takedown::Column::FeedId.is_null())
            .filter(benben::Column::UserId.eq(uid))
            .filter(benben::Column::Time.gte(one_year_ago.naive_utc()))
            .count(db)
            .await?;

        if one_year_count > 512 {
            512
        } else if one_year_count < 128 {
            128
        } else {
            one_year_count
        }
    };

    let feeds: Vec<benben::Model> = benben::Entity::find()
        .left_join(benben_takedown::Entity)
        .filter(benben_takedown::Column::FeedId.is_null())
        .filter(benben::Column::UserId.eq(uid))
        .order_by_desc(benben::Column::Time)
        .limit(limit)
        .all(db)
        .await?;

    let mut texts: Vec<String> = feeds
        .iter()
        .filter_map(|f| preprocess_text(&f.content))
        .collect();

    if texts.len() < 128 {
        let feeds = benben::Entity::find()
            .left_join(benben_takedown::Entity)
            .filter(benben_takedown::Column::FeedId.is_null())
            .filter(benben::Column::UserId.eq(uid))
            .order_by_desc(benben::Column::Time)
            .limit(256)
            .all(db)
            .await?;

        texts = feeds
            .iter()
            .filter_map(|f| preprocess_text(&f.content))
            .collect();
    }

    let count = texts.len() as u64;
    if count < 128 {
        return Err(AppError::InvalidParam(format!(
            "Insufficient valid posts: uid = {uid}, count = {count}, need >= 128"
        )));
    }

    Ok((texts, count))
}

// ==================== Text Preprocessing ====================

fn preprocess_text(content: &str) -> Option<String> {
    let text = content.split("|| @").next().unwrap_or(content).trim();
    if text.is_empty() {
        return None;
    }
    let chars: Vec<char> = text.chars().take(512).collect();
    Some(chars.into_iter().collect())
}

// ==================== Embedding ====================

fn compute_embeddings(
    state: &DuelState,
    texts: &[String],
) -> Result<Array2<f32>, Box<dyn std::error::Error>> {
    let embeddings = state.model.embed(texts)?;

    let n = embeddings.len();
    let d = state.model.dim();
    eprintln!("embeddings: n = {n}, d = {d}");
    let mut arr = Array2::zeros((n, d));
    for (i, emb) in embeddings.iter().enumerate() {
        for (j, &val) in emb.iter().enumerate() {
            arr[[i, j]] = val;
        }
    }
    Ok(arr)
}

// ==================== Topic Resonance ====================

fn compute_topic_resonance(a: &Array2<f32>, b: &Array2<f32>) -> serde_json::Value {
    let center_a = a.mean_axis(Axis(0)).unwrap();
    let center_b = b.mean_axis(Axis(0)).unwrap();

    let norm_a = center_a.mapv(|x| x * x).sum().sqrt();
    let norm_b = center_b.mapv(|x| x * x).sum().sqrt();

    if norm_a < 1e-6 || norm_b < 1e-6 {
        return json!(0.0);
    }

    let cosine = (center_a.dot(&center_b) / (norm_a * norm_b)).max(0.0);
    json!(cosine)
}

// ==================== Divergence ====================

fn compute_divergence(a: &Array2<f32>, b: &Array2<f32>) -> serde_json::Value {
    let center_a = a.mean_axis(Axis(0)).unwrap();
    let center_b = b.mean_axis(Axis(0)).unwrap();

    let n_a = a.nrows() as f32;
    let n_b = b.nrows() as f32;

    let diff_a = a - &center_a;
    let diff_b = b - &center_b;

    let var_a = diff_a.mapv(|x| x * x).sum_axis(Axis(1)).sum() / (n_a - 1.0).max(1.0);
    let var_b = diff_b.mapv(|x| x * x).sum_axis(Axis(1)).sum() / (n_b - 1.0).max(1.0);

    let ratio = var_a.max(1e-6) / var_b.max(1e-6);
    let log_ratio = ratio.ln().clamp(-3.0, 3.0);
    json!(log_ratio)
}

// ==================== Entropy ====================

fn compute_entropy(a: &Array2<f32>, b: &Array2<f32>) -> serde_json::Value {
    let (mean_a, std_a) = pairwise_cosine_stats(a);
    let (mean_b, std_b) = pairwise_cosine_stats(b);

    let mean_ratio = if mean_b > 1e-6 { mean_a / mean_b } else { 1.0 };
    let std_diff = (std_a - std_b).abs();

    json!({
        "mean_ratio": mean_ratio,
        "std_diff": std_diff,
    })
}

fn pairwise_cosine_stats(m: &Array2<f32>) -> (f32, f32) {
    let n = m.nrows();
    if n < 2 {
        return (0.0, 0.0);
    }

    let norms = m.mapv(|x| x * x).sum_axis(Axis(1)).mapv(f32::sqrt);
    let mut normalized = m.clone();
    for i in 0..n {
        let norm = norms[i];
        if norm > 1e-6 {
            normalized.row_mut(i).mapv_inplace(|x| x / norm);
        }
    }

    let sim = normalized.dot(&normalized.t());

    let mut dists: Vec<f32> = Vec::with_capacity(n * (n - 1) / 2);
    for i in 0..n {
        for j in (i + 1)..n {
            dists.push((1.0 - sim[[i, j]]).max(0.0));
        }
    }

    let mean = dists.iter().sum::<f32>() / dists.len() as f32;
    let variance = dists.iter().map(|&d| (d - mean).powi(2)).sum::<f32>() / dists.len() as f32;
    let std_dev = variance.sqrt();

    (mean, std_dev)
}

// ==================== SimHash ====================

fn compute_duplicate_rate(texts_a: &[String], texts_b: &[String]) -> f64 {
    let hashes_a: Vec<u64> = texts_a.iter().filter_map(|t| compute_simhash(t)).collect();
    let hashes_b: Vec<u64> = texts_b.iter().filter_map(|t| compute_simhash(t)).collect();

    if hashes_a.is_empty() || hashes_b.is_empty() {
        return 0.0;
    }

    let mut close = 0u64;
    let mut total = 0u64;

    for &ha in &hashes_a {
        for &hb in &hashes_b {
            let dist = (ha ^ hb).count_ones();
            if dist < 3 {
                close += 1;
            }
            total += 1;
        }
    }

    close as f64 / total as f64
}

fn compute_simhash(text: &str) -> Option<u64> {
    use std::hash::{Hash, Hasher};

    let words: Vec<&str> = JIEBA.cut(text, true);

    let mut counts: HashMap<&str, f32> = HashMap::new();
    for word in words {
        let w = word.trim();
        if w.len() < 2 {
            continue;
        }
        if STOP_WORDS.iter().any(|&sw| sw == w) {
            continue;
        }
        *counts.entry(w).or_insert(0.0) += 1.0;
    }

    if counts.is_empty() {
        return None;
    }

    let mut v: [f32; 64] = [0.0; 64];
    for (word, &weight) in &counts {
        let mut hasher = ahash::AHasher::default();
        word.hash(&mut hasher);
        let hash = hasher.finish();

        for i in 0..64 {
            if (hash >> i) & 1 == 1 {
                v[i] += weight;
            } else {
                v[i] -= weight;
            }
        }
    }

    let mut fingerprint: u64 = 0;
    for i in 0..64 {
        if v[i] > 0.0 {
            fingerprint |= 1 << i;
        }
    }

    Some(fingerprint)
}
