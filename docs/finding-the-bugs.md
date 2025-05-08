# Finding the Sync Bugs

This is a description of my exploration into the world of Mac OSX file system management and Cloud services. I learned about the following:

NOTE: this is a set of notes and may be incomplete and partially LLM sourced.

- New options for ls, to see attributes: `ls -ldO`
    - *com.apple.*: 
        - .provenance:
    - *dataless*
    - *cloud*

- Single user mode analysis and mounting
- iCloud cache management utilities

## Finding the Culprit

My journey began by not being able to execute `git status .`

Outline:
- git status not working but not bugging
- rsync stuck
- cp stuck


# Understanding BSD File Flags and Ghosted iCloud Files on macOS

Recently, I ran into a frustrating issue trying to copy files from a macOS system that were synced with iCloud. The copy operation would fail mysteriously, sometimes with errors like `fcopyfile: Resource deadlock avoided`, and other times would hang indefinitely. After digging into the problem, I discovered that **BSD file flags**, specifically the `dataless` flag, were at the heart of the issue.

## 📦 What Are BSD File Flags?

On macOS (and other BSD-derived systems), filesystems support internal flags beyond normal permissions like `rwx`. These flags are stored in the `st_flags` field of a file’s metadata and represent special properties that affect how the file behaves.

You can inspect these flags using:

```sh
ls -ldO <path>
```

The `-O` flag tells `ls` to show these special BSD flags. Here's an example of what you might see:

```sh
-rw-r--r--@ 1 user staff 1024 May  8 10:00 file.txt compressed,dataless
```

This tells us:

* `compressed`: The file is stored on disk in compressed form (APFS feature).
* `dataless`: The file **does not contain actual data locally** — it's a **ghost file**, typically offloaded to iCloud.

## 🌩️ Ghost Files: The `dataless` Flag

When iCloud's “Optimize Mac Storage” feature is enabled, macOS may automatically offload file contents to iCloud, replacing the file with a lightweight metadata stub. These stub files look like real files, but:

* **Cannot be read or copied** without triggering a cloud fetch.
* **Cause tools like `cp` or `rsync` to fail or hang** in restricted environments (e.g. Recovery Mode).
* Are identified by the **`dataless`** BSD flag.

### Detecting Ghost Files in Scripts

Instead of just checking file size or xattrs, you can reliably detect a ghost file like this:

```sh
ls -ldO <file> | grep dataless
```

Or in Rust, use `fstat()` to check if `st_flags & UF_DATALESS != 0`.

## 🧪 Combining with Extended Attributes

Some files also expose cloud metadata via extended attributes (xattrs), such as:

* `com.apple.cloud`
* `com.apple.declared`
* `com.apple.provenance`

While not as reliable for detecting `dataless` behavior, xattrs are useful for debugging how macOS is treating a file under the hood.

## ✅ Practical Tip

If you're building tools to work with files on macOS and want to avoid issues with iCloud placeholders:

* **Skip files marked `dataless` unless you can trigger a fetch.**
* **Use BSD flag checks (`UF_DATALESS`) rather than relying solely on xattrs.**
* Consider offering a way to **list and download** ghosted files proactively.

---

By understanding how macOS flags and manages these ghost files, you can build tools and workflows that are robust even in partially synced or restricted environments.

