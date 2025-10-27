pub fn data_limit(x: f64, min: f64, max: f64) -> f64 {
    if x < min {
        min
    } else if x > max {
        max
    } else {
        x
    }
}

#[inline(always)]
pub fn quicksort(arr: &mut [i32]) {
    let len = arr.len();
    if len <= 1 {
        return;
    }
    
    let pivot = arr[len - 1];
    let mut i = 0;
    
    for j in 0..len - 1 {
        if arr[j] <= pivot {
            arr.swap(i, j);
            i += 1;
        }
    }
    
    arr.swap(i, len - 1);
    
    if i > 0 {
        quicksort(&mut arr[0..i]);
    }
    if i + 1 < len {
        quicksort(&mut arr[i+1..]);
    }
}

