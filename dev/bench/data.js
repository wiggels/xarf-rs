window.BENCHMARK_DATA = {
  "lastUpdate": 1778553935848,
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
          "id": "9e90e1b1613aeb625b48a2236e6a66e3f84e4c87",
          "message": "Merge pull request #5 from wiggels/investigate-perf-regression-VBprw\n\nfix: raise bench alert threshold to 125% for shared runner noise",
          "timestamp": "2026-05-11T21:18:10-04:00",
          "tree_id": "3ae9af8cbe50bea4247c7c8155ced80f4871c4b0",
          "url": "https://github.com/wiggels/xarf-rs/commit/9e90e1b1613aeb625b48a2236e6a66e3f84e4c87"
        },
        "date": 1778549086869,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse/small_spam_str",
            "value": 22152,
            "range": "± 633",
            "unit": "ns/iter"
          },
          {
            "name": "parse/ddos_str",
            "value": 19558,
            "range": "± 391",
            "unit": "ns/iter"
          },
          {
            "name": "parse/large_spam_str",
            "value": 52808,
            "range": "± 537",
            "unit": "ns/iter"
          },
          {
            "name": "parse/small_spam_prevalued",
            "value": 17894,
            "range": "± 2385",
            "unit": "ns/iter"
          },
          {
            "name": "parse/small_spam_strict",
            "value": 24029,
            "range": "± 193",
            "unit": "ns/iter"
          },
          {
            "name": "parse/small_spam_show_missing",
            "value": 29724,
            "range": "± 320",
            "unit": "ns/iter"
          },
          {
            "name": "validate/small_spam_normal",
            "value": 14383,
            "range": "± 349",
            "unit": "ns/iter"
          },
          {
            "name": "validate/small_spam_strict",
            "value": 17905,
            "range": "± 778",
            "unit": "ns/iter"
          },
          {
            "name": "validate/ddos_normal",
            "value": 12489,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "generate/minimal_spam_build",
            "value": 16230,
            "range": "± 255",
            "unit": "ns/iter"
          },
          {
            "name": "generate/spam_build_with_evidence",
            "value": 19845,
            "range": "± 128",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha256/256",
            "value": 1901,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha512/256",
            "value": 3838,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha1/256",
            "value": 1419,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/md5/256",
            "value": 1591,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha256/4096",
            "value": 6464,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha512/4096",
            "value": 13146,
            "range": "± 404",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha1/4096",
            "value": 5756,
            "range": "± 180",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/md5/4096",
            "value": 10569,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha256/65536",
            "value": 79131,
            "range": "± 173",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha512/65536",
            "value": 161734,
            "range": "± 2005",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha1/65536",
            "value": 76304,
            "range": "± 5337",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/md5/65536",
            "value": 154212,
            "range": "± 495",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha256/1048576",
            "value": 1264669,
            "range": "± 38069",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha512/1048576",
            "value": 2654304,
            "range": "± 100583",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha1/1048576",
            "value": 1220911,
            "range": "± 6125",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/md5/1048576",
            "value": 2463651,
            "range": "± 2987",
            "unit": "ns/iter"
          },
          {
            "name": "v3/is_v3_report",
            "value": 74,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "v3/convert_v3_to_v4",
            "value": 9218,
            "range": "± 284",
            "unit": "ns/iter"
          },
          {
            "name": "v3/parse_v3_full_pipeline",
            "value": 33862,
            "range": "± 824",
            "unit": "ns/iter"
          },
          {
            "name": "registry/known_combination_hit",
            "value": 56,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry/known_combination_miss",
            "value": 50,
            "range": "± 2",
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
            "value": 22918,
            "range": "± 179",
            "unit": "ns/iter"
          },
          {
            "name": "round_trip/parse_then_serialize_large",
            "value": 82057,
            "range": "± 2072",
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
          "id": "ab10500f34e7289bbcefffc14d440e5e44a03652",
          "message": "Merge pull request #6 from wiggels/release-plz-2026-05-12T01-19-00Z",
          "timestamp": "2026-05-11T21:26:09-04:00",
          "tree_id": "063088b4e1164f72bd254c4d7efc2e55316d1288",
          "url": "https://github.com/wiggels/xarf-rs/commit/ab10500f34e7289bbcefffc14d440e5e44a03652"
        },
        "date": 1778549570437,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse/small_spam_str",
            "value": 21935,
            "range": "± 322",
            "unit": "ns/iter"
          },
          {
            "name": "parse/ddos_str",
            "value": 19806,
            "range": "± 131",
            "unit": "ns/iter"
          },
          {
            "name": "parse/large_spam_str",
            "value": 52453,
            "range": "± 1663",
            "unit": "ns/iter"
          },
          {
            "name": "parse/small_spam_prevalued",
            "value": 17551,
            "range": "± 2080",
            "unit": "ns/iter"
          },
          {
            "name": "parse/small_spam_strict",
            "value": 23874,
            "range": "± 258",
            "unit": "ns/iter"
          },
          {
            "name": "parse/small_spam_show_missing",
            "value": 29097,
            "range": "± 555",
            "unit": "ns/iter"
          },
          {
            "name": "validate/small_spam_normal",
            "value": 14326,
            "range": "± 241",
            "unit": "ns/iter"
          },
          {
            "name": "validate/small_spam_strict",
            "value": 18119,
            "range": "± 311",
            "unit": "ns/iter"
          },
          {
            "name": "validate/ddos_normal",
            "value": 12725,
            "range": "± 146",
            "unit": "ns/iter"
          },
          {
            "name": "generate/minimal_spam_build",
            "value": 16659,
            "range": "± 102",
            "unit": "ns/iter"
          },
          {
            "name": "generate/spam_build_with_evidence",
            "value": 20121,
            "range": "± 87",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha256/256",
            "value": 1830,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha512/256",
            "value": 3766,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha1/256",
            "value": 1369,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/md5/256",
            "value": 1674,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha256/4096",
            "value": 6901,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha512/4096",
            "value": 14140,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha1/4096",
            "value": 6204,
            "range": "± 100",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/md5/4096",
            "value": 11608,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha256/65536",
            "value": 87246,
            "range": "± 357",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha512/65536",
            "value": 180104,
            "range": "± 505",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha1/65536",
            "value": 83918,
            "range": "± 149",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/md5/65536",
            "value": 171429,
            "range": "± 2121",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha256/1048576",
            "value": 1378868,
            "range": "± 18111",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha512/1048576",
            "value": 2843146,
            "range": "± 10675",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha1/1048576",
            "value": 1331315,
            "range": "± 5109",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/md5/1048576",
            "value": 2736204,
            "range": "± 4006",
            "unit": "ns/iter"
          },
          {
            "name": "v3/is_v3_report",
            "value": 71,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "v3/convert_v3_to_v4",
            "value": 9345,
            "range": "± 4076",
            "unit": "ns/iter"
          },
          {
            "name": "v3/parse_v3_full_pipeline",
            "value": 33091,
            "range": "± 193",
            "unit": "ns/iter"
          },
          {
            "name": "registry/known_combination_hit",
            "value": 58,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry/known_combination_miss",
            "value": 52,
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
            "value": 22777,
            "range": "± 132",
            "unit": "ns/iter"
          },
          {
            "name": "round_trip/parse_then_serialize_large",
            "value": 84527,
            "range": "± 364",
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
          "id": "3b441320dc4a1a6e2d7be96a38d51d88d5df4036",
          "message": "Merge pull request #7 from wiggels/perf/hot-path-improvements",
          "timestamp": "2026-05-11T22:38:56-04:00",
          "tree_id": "31c41592d6fc559ec7fd8182d720cc5124639e7d",
          "url": "https://github.com/wiggels/xarf-rs/commit/3b441320dc4a1a6e2d7be96a38d51d88d5df4036"
        },
        "date": 1778553934995,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse/small_spam_str",
            "value": 18195,
            "range": "± 104",
            "unit": "ns/iter"
          },
          {
            "name": "parse/ddos_str",
            "value": 16178,
            "range": "± 298",
            "unit": "ns/iter"
          },
          {
            "name": "parse/large_spam_str",
            "value": 50303,
            "range": "± 310",
            "unit": "ns/iter"
          },
          {
            "name": "parse/small_spam_prevalued",
            "value": 14758,
            "range": "± 3920",
            "unit": "ns/iter"
          },
          {
            "name": "parse/small_spam_strict",
            "value": 20325,
            "range": "± 111",
            "unit": "ns/iter"
          },
          {
            "name": "parse/small_spam_show_missing",
            "value": 20432,
            "range": "± 192",
            "unit": "ns/iter"
          },
          {
            "name": "validate/small_spam_normal",
            "value": 11902,
            "range": "± 141",
            "unit": "ns/iter"
          },
          {
            "name": "validate/small_spam_strict",
            "value": 14663,
            "range": "± 166",
            "unit": "ns/iter"
          },
          {
            "name": "validate/ddos_normal",
            "value": 9971,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "generate/minimal_spam_build",
            "value": 12505,
            "range": "± 214",
            "unit": "ns/iter"
          },
          {
            "name": "generate/spam_build_with_evidence",
            "value": 15834,
            "range": "± 99",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha256/256",
            "value": 625,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha512/256",
            "value": 1269,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha1/256",
            "value": 577,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/md5/256",
            "value": 947,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha256/4096",
            "value": 5208,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha512/4096",
            "value": 10645,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha1/4096",
            "value": 5014,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/md5/4096",
            "value": 9940,
            "range": "± 197",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha256/65536",
            "value": 78487,
            "range": "± 243",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha512/65536",
            "value": 159595,
            "range": "± 626",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha1/65536",
            "value": 75799,
            "range": "± 186",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/md5/65536",
            "value": 153983,
            "range": "± 339",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha256/1048576",
            "value": 1267211,
            "range": "± 7941",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha512/1048576",
            "value": 2564541,
            "range": "± 59952",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/sha1/1048576",
            "value": 1225821,
            "range": "± 13738",
            "unit": "ns/iter"
          },
          {
            "name": "create_evidence/md5/1048576",
            "value": 2469468,
            "range": "± 3910",
            "unit": "ns/iter"
          },
          {
            "name": "v3/is_v3_report",
            "value": 74,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "v3/convert_v3_to_v4",
            "value": 6118,
            "range": "± 420",
            "unit": "ns/iter"
          },
          {
            "name": "v3/parse_v3_full_pipeline",
            "value": 23376,
            "range": "± 105",
            "unit": "ns/iter"
          },
          {
            "name": "registry/known_combination_hit",
            "value": 46,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry/known_combination_miss",
            "value": 55,
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
            "value": 19280,
            "range": "± 119",
            "unit": "ns/iter"
          },
          {
            "name": "round_trip/parse_then_serialize_large",
            "value": 80587,
            "range": "± 642",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}