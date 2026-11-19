# Install Starship

Set up [PrimitiveOS](https://github.com/DoanNguyenTrong/kindle-primitive) once with Primitive Launcher on a supported jailbroken Kindle. Open Store, refresh, and search for Starship. Tap Get and confirm the install.

## Install a GitHub package for local testing

1. Download the ARMHF ZIP and its SHA-256 file from the [Starship release page](https://github.com/DoanNguyenTrong/kindle-primitive-starship/releases/latest). Verify the checksum and extract the ZIP.
2. With Starship closed, connect by USB and copy the extracted `starship/` folder to the Kindle drive root. Copy `documents/Starship.sh` into the Kindle's existing `documents/` folder.
3. Eject the drive and open **Starship** in Launcher. Tap **Done** to return to Launcher.

The package targets hard-float Kindle devices with firmware 5.16.3 or newer and uses Launcher’s fonts.

## Upgrade from Starship 0.1.0

Update Starship in Store. For a manual upgrade, follow the USB steps above and merge the package into the existing `starship/` directory. Arcade missions use `starship/starship-arcade.json`. The previous turn-based mission and score are still in `starship/starship.json`.
