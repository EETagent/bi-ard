Dneska se vyměnil board za Arduino Esplora a zprovoznit Rust nebylo bez obtíží, takže řešení jsem udělal klasicky v cpp a tady se objeví možná později.

![IMG_9894](https://github.com/user-attachments/assets/4a330fdf-cc95-41de-8132-96196e12e628)

Každopádně zatím nějaké poznámky:

#### Setup

```sh
rustup target add avr-atmega32u4
```

```sh
cargo generate --git https://github.com/Rahix/avr-hal-template.git
```

Esplora je interně podobný board jako Leonardo, je potřeba vybrat tedy ten. Oba mají CPU atmega32u4


#### Flashování

Flashování (/dev/ttyACM0 bude na Linuxu) přes ravedude/avrdude bohužel vycrashuje protože Leonardo board musí být nejprve vyresetován. I přes hint od ravedude NESTAČÍ jednou zmáčknout reset tlačítko.

>arduino leonardo avrdude Error: initialization failed (rc = -1)

Dvě možné varianty:

1) python script co otevřením a zavřením spojení přes 1200 baudů zavede desku do reset módu (Takhle to dělá Arduino IDE před flashem)

```python3
import serial
import time
import sys

def reset_arduino_leonardo(port):
    """Reset Arduino Leonardo board before uploading firmware.

    This opens the specified port at 1200 baud, which triggers a soft reset
    on Leonardo and other ATmega32u4-based boards, putting them in bootloader mode.
    """
    try:
        # Open serial port at 1200 baud for soft reset
        ser = serial.Serial(port, 1200)
        ser.close()

        # Wait for the bootloader to become available
        print("Resetting Arduino Leonardo...")
        time.sleep(2)  # Give the board time to reset and enter bootloader mode

        return True
    except Exception as e:
        print(f"Error resetting Arduino: {e}")
        return False

if __name__ == "__main__":
    if len(sys.argv) > 1:
        port = sys.argv[1]
        reset_arduino_leonardo(port)
    else:
        print("Usage: python reset_leonardo.py <PORT>")
        print("Example: python reset_leonardo.py COM3")
```

2) Dvakrát rychle po sobě zmáčknout reset tlačítko. Někdy se to povede až na několikátý pokus. Musí rychle po sobě začít blikat ledky. Poté je jen KRÁTKÝ čas, kdy je možné desku naflashovat.

#### Upozornění!

Poté, co jsem flashnul přes avrdude zkompilovaný rustový kód na desku, zjistil jsem, že ji nemůžu připojit 😂. Arduino IDE spolu s C++ programem nahrává na desku i kód, který umožňuje Leonardu komunikovat přes sériové připojení, což v rustovém kódu, který se tam flashne, chybí. Není pak možné desku připojit znova a musí se flashnout přes možnost 2). Takový soft brick to je.
