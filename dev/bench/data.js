window.BENCHMARK_DATA = {
  "lastUpdate": 1778541092661,
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
      },
      {
        "commit": {
          "author": {
            "email": "hwigelsworth@vultr.com",
            "name": "Hunter Wigelsworth",
            "username": "wiggels"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "aae66be997a5e424bdc503f3e5c17dd83048c9e3",
          "message": "Merge pull request #3 from wiggels/release-plz-2026-05-11T22-31-52Z\n\nchore: release v0.1.2",
          "timestamp": "2026-05-11T16:05:04-07:00",
          "tree_id": "3ac2b87f7c0c741d793ffaa11113c4a6fc8e4c33",
          "url": "https://github.com/wiggels/xarf-rs/commit/aae66be997a5e424bdc503f3e5c17dd83048c9e3"
        },
        "date": 1778541092272,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse/small_spam_str",
            "value": 19782,
            "range": "± 83",
            "unit": "ns/iter"
          },
          {
            "name": "parse/ddos_str",
            "value": 17765,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "parse/large_spam_str",
            "value": 48004,
            "range": "± 201",
            "unit": "ns/iter"
          },
          {
            "name": "parse/small_spam_prevalued",
            "value": 17065,
            "range": "± 3890",
            "unit": "ns/iter"
          },
          {
            "name": "parse/small_spam_strict",
            "value": 21592,
            "range": "± 80",
            "unit": "ns/iter"
          },
          {
            "name": "parse/small_spam_show_missing",
            "value": 25925,
            "range": "± 97",
            "unit": "ns/iter"
          },
          {
            "name": "validate/small_spam_normal",
            "value": 13708,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "validate/small_spam_strict",
            "value": 16538,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "validate/ddos_normal",
            "value": 12071,
            "range": "± 131",
            "unit": "ns/iter"
          },
          {
            "name": "generate/minimal_spam_build",
            "value": 15227,
            "range": "± 268",
            "unit": "ns/iter"
          },
          {
            "name": "generate/spam_build_with_evidence",
            "value": 18195,
            "range": "± 130",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha256/256",
            "value": 1725,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha512/256",
            "value": 3523,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha1/256",
            "value": 1307,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/md5/256",
            "value": 1452,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha256/4096",
            "value": 6443,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha512/4096",
            "value": 12850,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha1/4096",
            "value": 5353,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/md5/4096",
            "value": 9708,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha256/65536",
            "value": 82016,
            "range": "± 101",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha512/65536",
            "value": 162223,
            "range": "± 3049",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha1/65536",
            "value": 71372,
            "range": "± 124",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/md5/65536",
            "value": 141627,
            "range": "± 405",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha256/1048576",
            "value": 1332496,
            "range": "± 11867",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha512/1048576",
            "value": 2587298,
            "range": "± 4766",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha1/1048576",
            "value": 1161650,
            "range": "± 1602",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/md5/1048576",
            "value": 2288429,
            "range": "± 3098",
            "unit": "ns/iter"
          },
          {
            "name": "v3/is_v3_report",
            "value": 61,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "v3/convert_v3_to_v4",
            "value": 7631,
            "range": "± 1713",
            "unit": "ns/iter"
          },
          {
            "name": "v3/parse_v3_full_pipeline",
            "value": 30144,
            "range": "± 81",
            "unit": "ns/iter"
          },
          {
            "name": "registry/known_combination_hit",
            "value": 49,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry/known_combination_miss",
            "value": 48,
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
            "value": 21013,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "round_trip/parse_then_serialize_large",
            "value": 76228,
            "range": "± 189",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}