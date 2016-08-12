fn spawn_for(host: &Host,
             hash: &str,
             config: &Config,
             output: &Output)
             -> (String, JoinHandle<bool>) {

    let thread_host = host.clone();
    let thread_hash = hash.to_string();
    let thread_cmds = config.build.clone();
    let thread_output = output.clone();

    let join_handle = thread::spawn(move || {
        thread_host.run(&thread_hash, &thread_cmds, &thread_output)
    });

    (host.name.to_string(), join_handle)
}

fn join_all(threads: Vec<(String, JoinHandle<bool>)>) -> Vec<(String, bool)> {
    threads.into_iter().map(|pair| (pair.0, pair.1.join().unwrap_or(false))).collect()
}

fn print_summary(task: &str, results: &[(String, bool)], output: &Output) {
    println!("\n---------- Summary ----------\n");

    for result in results {
        let log = Log::new(&result.0, output);

        if result.1 {
            log.success(format!("{} succeeded", task));
        } else {
            log.error(format!("{} failed", task));
        }
    }
}

fn succeeded(results: &[(String, bool)]) -> bool {
    results.iter().all(|pair| pair.1)
}

pub fn spawn(hosts: &[Host],
             hash: Option<String>,
             commands: &[String],
             output: &Output)
             -> Result<i32, Box<Error>> {

    let results = join_all(hosts.iter()
                                .map(|host| spawn_for(host, &hash, &config, &output))
                                .collect());

    if results.len() > 1 {
        print_summary(&results, &output);
    }

    if succeeded(&results) {
        Ok(0)
    } else {
        Ok(1)
    }
}
