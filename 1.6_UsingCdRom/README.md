# PSX Using the CD-ROM

The original tutorial is [archived here](https://web.archive.org/web/20240916202225/http://lameguy64.net/tutorials/pstutorials/chapter1/6-cdreading.html).

[MKPSXISO](https://github.com/lameguy64/mkpsxiso) is required to create the PSX CD image which can then be loaded in an emulator (mednafen, or any other), or burnt to a CD-R to run on
an original PSX via the usual methods of running homebrew discs. This super simple tech demo is probably not worth using up a CD-R for. You have been warned.

**Don't** use `cargo psx run` for this example. The compile executable needs to be combined with the TIM image to the ISO.

The executable cannot locate the image unless they are both located on the CDROM filesystem.
The program simply hangs on a blank screen if the file cannot be located. There does not appear to be a timeout (but it might depend on the emulator).
Specific errors will cause the code to panic!.


To build, package and generate a .BIN .CUE CD format, and load the CD in mednafen emulator,
run:

```
cargo psx build

mkpsxiso mkpsxiso.xml

mednafen cdtutorial.cue
```

## Notes

Loading the TIM image file to memory using psx-sdk-rs's `File::<CDROM>::open()` was straightforward.

*Getting* the image from memory to VRAM was harder.

psx-sdk-rs has a helpful macro `include_tim!` for including a TIM binary from file at compile time, and generating all the code to load it to VRAM
(using consts), which will return a `LoadedTIM` from the framebuffer.

Most of the code to do this is wrapped in the macro, after the binary include, so we can't reuse it without having the TIM included in our executable,
which we are avoiding in this example.

I had to copy out most of the code inside `include_tim!` to send an arbitrary memory region containing full TIM data populated at runtime to get the same `LoadedTIM` back.


### **TODO:**
* This code has too many hardcoded values, and the whole load-image-in-memory-to-VRAM should be moved to a more general helper function, but that should probably live in the sdk.


### Checking whether our TIM file is compiled into the executable:

```
# the macro include_tim! does NOT include the entire TIM, it excludes the header...
# search for the data chunk:
LANG=C grep -obUaPH "\x80\x02\x30\x00\x10\x00\x40" 1.5_FixedPointMath/target/mipsel-sony-psx/release/*.exe  #  expect a match
LANG=C grep -obUaPH "\x80\x02\x30\x00\x10\x00\x40" 1.6*/target/mipsel-sony-psx/release/*.exe                #  expect NO match
```
