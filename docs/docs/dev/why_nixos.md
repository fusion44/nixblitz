# Why NixOS?

You might be used to "installing" software or services on systems like Ubuntu or CentOS. On those systems, "installation" typically means:

1.  **Executing a script or command:** `sudo apt install my-service` or `sudo systemctl enable my-service`.
2.  **Making in-place changes:** Files are copied to specific directories (`/usr/bin`, `/etc/my-service/`), and system configuration files are modified directly.
3.  **Potential for state drift:** Over time, with many installs, uninstalls, and updates, the system can accumulate leftover files, conflicting dependencies, or an inconsistent state.

**On NixBlitz, which leverages NixOS, our approach is fundamentally different. Instead of "installing," you are _declaring_ or _enabling_ our services as part of your system's overall configuration.**

Think of it like this:

**Traditional "Installation" (e.g., Ubuntu):** You're taking a hammer and nails and directly attaching a new component to your existing, running system. If you want to remove it, you have to carefully pry it off, hoping you don't leave any holes or damage other parts.

**NixOS "Declaration/Enabling" (Our Approach):** Instead of direct modification, you're **updating a master blueprint** for your entire system. This blueprint precisely describes _everything_ your system should be, including the presence and configuration of any services.

When you tell NixOS to "enable" our service:

1.  **You edit your system's configuration file (the blueprint).** You add a line like `services.bitcoind.enable = true;` and specify its desired settings.
2.  **NixOS then _builds a brand new, complete system_ based on this updated blueprint.** This new system is perfectly consistent and includes our service exactly as you specified, along with all its required dependencies.
3.  **Your system then atomically _switches_ to this newly built configuration.** This switch is instant and guaranteed to succeed, or it will automatically revert to your previous working configuration. Although, sometimes a switch succeeds but a service might not start due to a missonfiguration.

**Why this matters and why we avoid "installing":**

- **Reproducibility:** Your system configuration is now entirely defined by a single file which can import other files. You can recreate your exact system, including our services, anywhere, anytime, just by using that configuration file. There's no guesswork about what was "installed" or how.
- **Reliability & Rollbacks:** Because NixOS builds a _new_ system state and switches to it, if anything goes wrong, you can instantly revert to the _previous working system state_ without any complex uninstall or troubleshooting. This reduces downtime. You're never modifying a live, running system; you're always swapping in a new, verified one.
- **Isolation:** Our services and their dependencies are precisely packaged and isolated from the rest of your system. This eliminates "dependency hell" – where different applications need conflicting versions of the same library – making your system more robust.

### A Note on Stateful Data: The Caveat

While NixOS excels at managing your system's _declarative configuration_ (the applications, services, and their settings), it's important to understand a key distinction: **stateful data is not part of this blueprint.**

This means that if our service, or any other application on your NixOS system, generates or relies on persistent data (like databases, user files in `/home`, logs in `/var/log`, or runtime data in `/var/lib`), that data is _not_ automatically managed or recreated by the NixOS configuration itself.

**For example:**

- If you enable a database service, the database _software_ is declaratively configured. However, the actual database files (e.g., your users' data, posts, etc.) will reside in a stateful location like `/var/lib/postgresql/data`.
- If you were to take your NixOS configuration file and apply it to a brand new, empty machine, NixOS would perfectly recreate the software environment for our service, but it would **not** migrate any existing database content, user files, or application-specific data.

**Therefore, while the system itself is reproducible from its configuration, you remain responsible for backing up and restoring any critical _stateful data_ specific to our services or your applications when migrating or rebuilding your system.** This is a common pattern in any system administration, but it's important to differentiate it from the purely declarative nature of the NixOS system configuration itself.

---

So, when you're setting up our services, you're not "installing" in the traditional sense of making destructive, in-place changes. You are **declaring your desired system state** and letting NixOS build and switch to that perfectly consistent state for you. This approach ensures a far more stable, reproducible, and reliable experience for running [Your Software Name], with the understanding that persistent data needs its own management strategy.
