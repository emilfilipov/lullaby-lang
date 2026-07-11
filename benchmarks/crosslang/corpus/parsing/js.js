// Cross-language parsing suite (JavaScript). Real-world parsing over strings and
// int arrays mirroring ../lullaby.lby. `eval_rpn` takes an array plus a length.
// Invalid numeric input returns -1 (or 0 for the signed parser, whose full range
// includes -1). See ../SPEC.md.

function parse_uint(s) {
  if (s.length === 0) {
    return -1;
  }
  let val = 0;
  for (let i = 0; i < s.length; i++) {
    const o = s.charCodeAt(i);
    if (o < 48 || o > 57) {
      return -1;
    }
    val = val * 10 + (o - 48);
  }
  return val;
}

function parse_int_signed(s) {
  if (s.length === 0) {
    return 0;
  }
  const neg = s[0] === "-";
  const start = neg ? 1 : 0;
  if (start === s.length) {
    return 0;
  }
  let val = 0;
  for (let i = start; i < s.length; i++) {
    const o = s.charCodeAt(i);
    if (o < 48 || o > 57) {
      return 0;
    }
    val = val * 10 + (o - 48);
  }
  return neg ? -val : val;
}

function is_valid_int(s) {
  if (s.length === 0) {
    return 0;
  }
  const start = s[0] === "-" ? 1 : 0;
  if (start === s.length) {
    return 0;
  }
  for (let i = start; i < s.length; i++) {
    const o = s.charCodeAt(i);
    if (o < 48 || o > 57) {
      return 0;
    }
  }
  return 1;
}

function count_fields(s, sep) {
  const target = sep[0];
  let count = 0;
  for (let i = 0; i < s.length; i++) {
    if (s[i] === target) {
      count += 1;
    }
  }
  return count + 1;
}

function nth_field_len(s, sep, nth) {
  const target = sep[0];
  let field = 0;
  let cur = 0;
  let result = -1;
  for (let i = 0; i < s.length; i++) {
    if (s[i] === target) {
      if (field === nth) {
        result = cur;
      }
      field += 1;
      cur = 0;
    } else {
      cur += 1;
    }
  }
  if (field === nth) {
    result = cur;
  }
  return result;
}

function count_lines(s) {
  if (s.length === 0) {
    return 0;
  }
  let count = 0;
  for (let i = 0; i < s.length; i++) {
    if (s[i] === "\n") {
      count += 1;
    }
  }
  return count + 1;
}

function strip_leading_zeros_len(s) {
  let i = 0;
  while (i < s.length && s[i] === "0") {
    i += 1;
  }
  return s.length - i;
}

function eval_rpn(tokens, n) {
  if (n === 0) {
    return 0;
  }
  const stack = [];
  for (let i = 0; i < n; i++) {
    const t = tokens[i];
    if (t >= 0) {
      stack.push(t);
    } else {
      const b = stack.pop();
      const a = stack.pop();
      const op = -t;
      let r;
      if (op === 1) {
        r = a + b;
      } else if (op === 2) {
        r = a - b;
      } else if (op === 3) {
        r = a * b;
      } else {
        r = Math.trunc(a / b);
      }
      stack.push(r);
    }
  }
  return stack[0];
}

function count_digits_in(s) {
  let count = 0;
  for (let i = 0; i < s.length; i++) {
    const o = s.charCodeAt(i);
    if (o >= 48 && o <= 57) {
      count += 1;
    }
  }
  return count;
}

function count_words(s) {
  return s.split(/\s+/).filter(function (w) { return w.length > 0; }).length;
}

function hex_to_int(s) {
  if (s.length === 0) {
    return -1;
  }
  let val = 0;
  for (let i = 0; i < s.length; i++) {
    const o = s.charCodeAt(i);
    let d;
    if (o >= 48 && o <= 57) {
      d = o - 48;
    } else if (o >= 97 && o <= 102) {
      d = o - 97 + 10;
    } else if (o >= 65 && o <= 70) {
      d = o - 65 + 10;
    } else {
      return -1;
    }
    val = val * 16 + d;
  }
  return val;
}

function bin_to_int(s) {
  if (s.length === 0) {
    return -1;
  }
  let val = 0;
  for (let i = 0; i < s.length; i++) {
    const c = s[i];
    if (c === "0") {
      val = val * 2;
    } else if (c === "1") {
      val = val * 2 + 1;
    } else {
      return -1;
    }
  }
  return val;
}

function roman_value(c) {
  switch (c) {
    case "I": return 1;
    case "V": return 5;
    case "X": return 10;
    case "L": return 50;
    case "C": return 100;
    case "D": return 500;
    case "M": return 1000;
    default: return 0;
  }
}

function roman_to_int(s) {
  let total = 0;
  const n = s.length;
  for (let i = 0; i < n; i++) {
    const v = roman_value(s[i]);
    if (i + 1 < n && v < roman_value(s[i + 1])) {
      total -= v;
    } else {
      total += v;
    }
  }
  return total;
}

function char_class_count(s) {
  let count = 0;
  for (let i = 0; i < s.length; i++) {
    const o = s.charCodeAt(i);
    if ((o >= 65 && o <= 90) || (o >= 97 && o <= 122)) {
      count += 1;
    }
  }
  return count;
}

function main() {
  const rpn = [3, 4, -1, 5, -3];
  console.assert(parse_uint("01234") === 1234);
  console.assert(parse_int_signed("-42") === -42);
  console.assert(is_valid_int("-42") === 1);
  console.assert(count_fields("a,b,c,d", ",") === 4);
  console.assert(nth_field_len("a,bb,ccc", ",", 2) === 3);
  console.assert(count_lines("a\nb") === 2);
  console.assert(strip_leading_zeros_len("00042") === 2);
  console.assert(eval_rpn(rpn, 5) === 35);
  console.assert(count_digits_in("ab12cd34") === 4);
  console.assert(count_words("the quick brown fox") === 4);
  console.assert(hex_to_int("1a2f") === 6703);
  console.assert(bin_to_int("101101") === 45);
  console.assert(roman_to_int("MCMXCIV") === 1994);
  console.assert(char_class_count("abc123XYZ") === 6);
  console.log("ok");
}

main();
