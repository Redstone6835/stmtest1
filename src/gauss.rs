use crate::utils::quicksort;

pub fn gauss_filter(x: &[i32], n: usize) -> f64 {
    if n <= 0 {
        return 0.0;
    }
    if n == 1 {
        return x[0] as f64;
    }
    
    // 创建一个临时数组来存储排序后的数据
    let mut sorted = [0i32; 64]; // 假设最大长度为64，根据实际需求调整
    for i in 0..n {
        sorted[i] = x[i];
    }
    quicksort(&mut sorted[0..n]);

    // --- 初始估计 ---
    let mut sum: f64 = 0.0;
    for i in 0..n {
        sum += sorted[i] as f64;
    }
    let mut mu = sum / (n as f64);

    let mut var: f64 = 0.0;
    for i in 0..n {
        let diff = sorted[i] as f64 - mu;
        var += diff * diff;
    }
    var /= (n - 1) as f64;
    let mut sigma: f64 = libm::sqrt(var);

    if sigma < 1e-6 {
        sigma = 1e-6;
    }

    // --- 迭代更新 ---
    let max_iter = 30;
    let tol = 1e-6;

    for _iter in 0..max_iter {
        let mut sum_w: f64 = 0.0;
        let mut sum_wx: f64 = 0.0;

        for i in 0..n {
            let diff: f64 = (sorted[i] as f64 - mu) / sigma;
            let exponent: f64 = -0.5 * diff * diff;
            let mut weight = libm::exp(exponent);
            if weight < 1e-10 {
                weight = 1e-10;
            }
            
            sum_w += weight;
            sum_wx += weight * sorted[i] as f64;
        }

        let new_mu = sum_wx / sum_w;
        if libm::fabs(new_mu - mu) < tol {
            mu = new_mu;
            break;
        }

        mu = new_mu;

        // 更新 sigma
        let mut sum_var = 0.0;
        let mut sum_var_w = 0.0;
        for i in 0..n {
            let diff = sorted[i] as f64 - mu;
            let exponent = -0.5 * (diff / sigma) * (diff / sigma);
            let mut weight = libm::exp(exponent);
            if weight < 1e-10 {
                weight = 1e-10;
            }

            sum_var += weight * diff * diff;
            sum_var_w += weight;
        }
        sigma = libm::sqrt(sum_var / sum_var_w);
        if sigma < 1e-6 {
            sigma = 1e-6;
        }
    }

    return mu;
}
