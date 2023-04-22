- kezeli a Dolby XML input baromságait a háttérben, rendes CLI felületet adva
- átkonvertálja az inputokat RF64-re, amit már DEE is tud kezelni
  - a bitmélységet, csatornák számát és egyéb infókat a forrásból parse-olja
- minden input fájlhoz generál egy XML fájlt a beállítások alapján
- a script thread poolingot használ batch encodingra (lásd config)
- támogatja a WSL útvonalak konvertálását a DEE Windows verziójához (lásd config)
- hibás bitráta megadása esetén kiválasztja a legközelebbi megengedettet
- automatikus mintavételezésiráta-konvertálás ffmpeg soxr resamplerét használva nem támogatott mintavételezési ráta esetén
  - DD/DDP/AC4 esetén a mintavételezési rátát 48 kHz-re konvertálja
  - TrueHD esetén a mintavételezési rátát 48 kHz-re konvertálja, ha a forrásé kisebb mint 72 kHz, fölötte 96 kHz-re
- automatikus csatornafelcserélés 7.1-es forrásoknál (DEE valamiért megcseréli az Ls, Rs csatornákat az Lrs, Rrs-sel)
- automatikus dialnorm beállítás
- automatikusan kompenzálja a DEE 256 mintavételezés eltolását (DD és DDP encoding esetén)
- ellenőrzi, hogy az ideiglenes fájl létezik-e már, lehetővé téve különböző formátumok/bitráták kódolásását egyetlen ideiglenes fájl készítésével, például:\
  `deew -f dd -b 448 -i input -k`\
  `deew -f dd -b 640 -i input -k`\
  `deew -f ddp -i input`
- akár videó inputokkal is működik (az első audió kerül kiválasztásra)
- csicsás terminálkimenet rich használatával
- sokoldalú delay opció, ami támogat ms, s and és frame@fps formát is
- delay parse-olása fájlnévből