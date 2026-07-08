pub mod duel;
pub mod embedding;

pub use embedding::DuelState;

export_routes_group!(duel);

// 返回结构
// {
//   "topic_resonance": 0.335,          // f64, 余弦相似度 0~1
//   "divergence": -0.758,              // f64, v_a/v_b 的对数 -3~3
//   "entropy": {
//     "mean_ratio": 1.002,             // f64, 组内余弦距离均值比
//     "std_diff": 0.006                // f64, 组内余弦距离标准差差值
//   },
//   "duplicate_rate": 0.0001           // f64, SimHash 近重复率 0~1
// }
// 前端渲染规则
// topic_resonance（话题重合度）
// > 0.8   → "高度契合"
// > 0.5   → "中度共鸣"
// ≤ 0.5   → "差异较大"
// divergence（发散度对比）
// 正值    → A 更发散，百分比 = (e^|v| - 1) × 100
// 负值    → B 更发散
// |v| > 0.5 → 显示"X比Y发散N%"
// |v| ≤ 0.5 → "发散度相近"
// entropy.mean_ratio（表达多样性）
// > 1.2   → "A表达更丰富"
// < 0.8   → "B表达更丰富"
// [0.8, 1.2] → "多样性相近"
// entropy.std_diff（表达稳定性）
// > 0.05  → "稳定性有差异"
// ≤ 0.05  → "稳定性相近"
// duplicate_rate（SimHash 重复率）
// > 0.3   → 警告"存在较多相似用词"
// ≤ 0.3   → 正常，不提示
// 展示值  → (rate × 100).toFixed(2) + "%"
// sample_count_a/b → 直接展示"采样 N 条"
