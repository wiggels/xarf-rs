window.BENCHMARK_DATA = {
  "lastUpdate": 1778540410896,
  "repoUrl": "https://github.com/wiggels/xarf-rs",
  "entries": {
    "Benchmark": [
      {
        "commit": {
          "author": {
            "email": "hwigelsworth@vultr.com",
            "name": "hwigelsworth",
            "username": "wiggels"
          },
          "committer": {
            "email": "hwigelsworth@vultr.com",
            "name": "hwigelsworth",
            "username": "wiggels"
          },
          "distinct": true,
          "id": "bc353d259035e392a8aae8141652bda73e741303",
          "message": "fix: bootstrap gh-pages",
          "timestamp": "2026-05-11T18:53:06-04:00",
          "tree_id": "33acbb5101f917fed5b6d34260fdfb47b55847c8",
          "url": "https://github.com/wiggels/xarf-rs/commit/bc353d259035e392a8aae8141652bda73e741303"
        },
        "date": 1778540409989,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse/small_spam_str",
            "value": 16643,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "parse/ddos_str",
            "value": 14800,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "parse/large_spam_str",
            "value": 39921,
            "range": "± 330",
            "unit": "ns/iter"
          },
          {
            "name": "parse/small_spam_prevalued",
            "value": 13805,
            "range": "± 3895",
            "unit": "ns/iter"
          },
          {
            "name": "parse/small_spam_strict",
            "value": 18478,
            "range": "± 72",
            "unit": "ns/iter"
          },
          {
            "name": "parse/small_spam_show_missing",
            "value": 21917,
            "range": "± 183",
            "unit": "ns/iter"
          },
          {
            "name": "validate/small_spam_normal",
            "value": 11244,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "validate/small_spam_strict",
            "value": 14122,
            "range": "± 121",
            "unit": "ns/iter"
          },
          {
            "name": "validate/ddos_normal",
            "value": 9925,
            "range": "± 130",
            "unit": "ns/iter"
          },
          {
            "name": "generate/minimal_spam_build",
            "value": 12722,
            "range": "± 108",
            "unit": "ns/iter"
          },
          {
            "name": "generate/spam_build_with_evidence",
            "value": 15128,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha256/256",
            "value": 1447,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha512/256",
            "value": 2944,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha1/256",
            "value": 1065,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/md5/256",
            "value": 1296,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha256/4096",
            "value": 5456,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha512/4096",
            "value": 11127,
            "range": "± 205",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha1/4096",
            "value": 4902,
            "range": "± 121",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/md5/4096",
            "value": 9139,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha256/65536",
            "value": 68025,
            "range": "± 165",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha512/65536",
            "value": 139772,
            "range": "± 233",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha1/65536",
            "value": 64969,
            "range": "± 174",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/md5/65536",
            "value": 133021,
            "range": "± 122",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha256/1048576",
            "value": 1074252,
            "range": "± 4551",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha512/1048576",
            "value": 2209419,
            "range": "± 4090",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha1/1048576",
            "value": 1033971,
            "range": "± 1480",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/md5/1048576",
            "value": 2126519,
            "range": "± 21077",
            "unit": "ns/iter"
          },
          {
            "name": "v3/is_v3_report",
            "value": 53,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "v3/convert_v3_to_v4",
            "value": 7386,
            "range": "± 6372",
            "unit": "ns/iter"
          },
          {
            "name": "v3/parse_v3_full_pipeline",
            "value": 25394,
            "range": "± 91",
            "unit": "ns/iter"
          },
          {
            "name": "registry/known_combination_hit",
            "value": 44,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry/known_combination_miss",
            "value": 41,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry/compile_master_validator",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "round_trip/parse_then_serialize_small",
            "value": 17173,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "round_trip/parse_then_serialize_large",
            "value": 64701,
            "range": "± 335",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}