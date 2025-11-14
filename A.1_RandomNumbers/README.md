# A.1: Random Numbers

Uses controller input and a low level system timer to seed the `psx::sys::rng` `RNG` to generate random `u8`s on controller 'X' button press.

I resorted to using an `asm!` macro to read the timer value back from v0(r2) after calling the kernel function `psx_get_timer()`

Perhaps I'm missing something obvious, but I couldn't see a higher level way to get the value back from the register.


There's also some basic button debounce code in here that uses draw_sync / vblank_wait for timing.

**TODO:** Investigate whether the draw_sync is taking too long with controllers enabled. Redraw times do seem to be longer than I'd expect,
the debounce is currently at 5 draw loops, and that's quite noticble. I was expecting to have to set it considerably higher to notice the delay.
