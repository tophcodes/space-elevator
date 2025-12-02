# Space Elevator

Space elevator is a GUI-based configuration tool for 6 degrees-of-freedom (6dof) input devices, mostly known as space mice. It communicates with a separately running `spacenavd` daemon (a user-space 6dof device driver) via `libspnav`.

There already exists a `spacenav-sys` and `spacenav` crate, however this looks to be unmaintained and is built against a very old version of libspnav. In order for `space-elevator` to communicate with the driver, this project includes new rust bindings for libspnav under the names `spnav`/`spnav-sys`.

For more info on the spacenav project (spacenavd and libspnav), visit their website: http://spacenav.sourceforge.net/

## Implemented devices

Currently the following devices are supported:
- [3D Connexion SpaceMouse Enterprise](https://3dconnexion.com/us/product/spacemouse-enterprise/)

## Application support

The following applications are supported:
- [Onshape](https://onshape.com) via the _Space Elevator_ browser extension

### Limitations

> Since the 3Dconnexion driver is practically unmaintained for the last decade, and never really worked very well to begin with (which was what led to the development of spacenavd), and also since the magellan protocol introduces an unnecessary dependency to the X window system, and provides very rudimentary functionality, new applications are encouraged to use `spnav_open`.
> -- [libspnav manual](https://spacenav.sourceforge.net/man_libspnav/#about-libspnav)

For this reason, the spnav/spnav-sys wrappers are not currently built against libspnav with X11 support enabled and therefore also do not expose any X11-related functionality.
