# cli

[![CI](https://github.com//cli/workflows/CI/badge.svg)](https://github.com//cli/actions)

A CLI interface to the RaspiBlitz project.


        let s = BitcoinDaemonService::default();
        let stringgg = match serde_json::to_string_pretty(&s) {
            Ok(value) => value,
            Err(_) => todo!(),
        };

        let path = Path::new("results.json");
        let display = path.display();
        let mut file = match File::create(path) {
            Err(e) => panic!("Couldn't create {}: {}", display, e),
            Ok(file) => file,
        };
        let _ = file.write_all(&stringgg.into_bytes());

