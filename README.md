### Limitations

> Since the 3Dconnexion driver is practically unmaintained for the last decade, and never really worked very well to begin with (which was what led to the development of spacenavd), and also since the magellan protocol introduces an unnecessary dependency to the X window system, and provides very rudimentary functionality, new applications are encouraged to use `spnav_open`.
> -- (libspnav manual)[https://spacenav.sourceforge.net/man_libspnav/#about-libspnav]

For this reason, the spnav/spnav-sys wrappers are not currently built against libspnav with X11 support enabled and therefore also do not expose any X11-related functionality.
