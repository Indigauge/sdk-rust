/// Bucket CPU core counts into coarse ranges for privacy.
pub fn bucket_cores(n: u32) -> &'static str {
  match n {
    0..=2 => "1-2",
    3..=4 => "3-4",
    5..=8 => "6-8",
    _ => ">8",
  }
}

/// Bucket RAM in GB into coarse ranges for privacy.
pub fn bucket_ram_gb(gb: u32) -> &'static str {
  match gb {
    0..=4 => "<=4",
    5..=8 => "6-8",
    9..=16 => "12-16",
    _ => ">16",
  }
}

/// Coarsen CPU model strings into privacy-safe categories.
pub fn coarsen_cpu_name(cpu_raw: &str) -> Option<String> {
  let name = cpu_raw.to_ascii_lowercase();

  if name.contains("apple m1") {
    return Some("Apple M1".into());
  } else if name.contains("apple m2") {
    return Some("Apple M2".into());
  } else if name.contains("apple m3") {
    return Some("Apple M3".into());
  }

  if name.contains("intel") {
    if name.contains("celeron") {
      return Some("Intel Celeron".into());
    }
    if name.contains("pentium") {
      return Some("Intel Pentium".into());
    }
    if name.contains("xeon") {
      return Some("Intel Xeon".into());
    }
    if name.contains("atom") {
      return Some("Intel Atom".into());
    }
    if name.contains("core") {
      if name.contains("i3") {
        return Some(generation_label("Intel i3", &name));
      } else if name.contains("i5") {
        return Some(generation_label("Intel i5", &name));
      } else if name.contains("i7") {
        return Some(generation_label("Intel i7", &name));
      } else if name.contains("i9") {
        return Some(generation_label("Intel i9", &name));
      } else {
        return Some("Intel Core (Other)".into());
      }
    }
    return Some("Intel (Other)".into());
  }

  if name.contains("amd") {
    if name.contains("ryzen") {
      if name.contains("threadripper") {
        return Some("AMD Ryzen Threadripper".into());
      }
      if let Some(generation) = extract_ryzen_gen(&name) {
        return Some(format!("AMD Ryzen {} Series", generation));
      }
      return Some("AMD Ryzen".into());
    }
    if name.contains("epyc") {
      return Some("AMD EPYC".into());
    }
    if name.contains("athlon") {
      return Some("AMD Athlon".into());
    }
    return Some("AMD (Other)".into());
  }

  if name.contains("arm") || name.contains("cortex") {
    return Some("ARM (Generic)".into());
  }

  None
}

fn generation_label(prefix: &str, name: &str) -> String {
  if let Some(generation) = extract_generation(name) {
    format!("{} {}th Gen", prefix, generation)
  } else {
    prefix.to_string()
  }
}

fn extract_generation(name: &str) -> Option<u8> {
  if let Some(pos) = name.find("th gen") {
    let before = &name[..pos];
    let digits: String = before.chars().rev().take_while(|c| c.is_ascii_digit()).collect();
    if !digits.is_empty() {
      return digits.chars().rev().collect::<String>().parse().ok();
    }
  }
  if let Some(pos) = name.find("i7-")
    && name.len() > pos + 3
    && let Some(d) = name[pos + 3..].chars().next()
  {
    return d.to_digit(10).map(|v| v as u8);
  }
  None
}

fn extract_ryzen_gen(name: &str) -> Option<String> {
  let parts: Vec<&str> = name.split_whitespace().collect();
  for (i, part) in parts.iter().enumerate() {
    if *part == "ryzen" && i + 2 < parts.len() {
      let next = parts[i + 2];
      if let Some(first_digit) = next.chars().next()
        && first_digit.is_ascii_digit()
      {
        return Some(format!("{}000", first_digit));
      }
    }
  }
  None
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn buckets_cores_and_ram() {
    assert_eq!(bucket_cores(1), "1-2");
    assert_eq!(bucket_ram_gb(16), "12-16");
  }

  #[test]
  fn coarsens_cpu_names() {
    assert_eq!(coarsen_cpu_name("11th Gen Intel Core i7"), Some("Intel i7 11th Gen".into()));
    assert_eq!(coarsen_cpu_name("AMD Ryzen 7 5800X"), Some("AMD Ryzen 5000 Series".into()));
  }
}
