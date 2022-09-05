- kezeli a Dolby XML input baromságait a háttérben, rendes CLI felületet adva
- átkonvertálja az inputokat rf64-re, amit már DEE is tud kezelni
  - a bitmélységet, csatornák számát és egyéb infókat a forrásból parse-olja
- minden input fájlhoz generál egy XML fájlt a beállítások alapján
- a script thread poolingot használ batch encodingra (lásd config)
- támogatja a WSL útvonalak konvertálását a DEE Win verziójához (lásd config)
- hibás bitráta megadása esetén kiválasztja a legközelebbi megengedettet
- automatikus mintavételezésiráta-konvertálás ffmpeg soxr resamplerét használva nem támogatott mintavételezési ráta esetén
  - dd/dddp esetén a mintavételezési rátát 48 000-re konvertálja
  - thd esetén a mintavételezési rátát 48 000-re konvertálja, ha a forrásé kisebb mint 72 000, fölötte 96 000-re
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