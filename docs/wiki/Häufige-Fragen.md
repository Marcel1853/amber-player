# Häufige Fragen

## Allgemein

**Kostet Amber Player etwas? Gibt es Werbung?**
Nein und nein. Amber Player ist kostenlos und werbefrei – und bleibt es.

**Sammelt Amber Player Daten?**
Nein. Es gibt kein Konto und keine Statistik. Ins Internet geht nur, was du nutzt: die
Cover-Suche (sendet Interpret, Titel, Album), die Radio-Senderliste und die Radiostreams.

**Gibt es Amber Player für Windows, macOS oder iPhone?**
Derzeit nicht. Es gibt Pakete für Linux und eine App für Android.

## Bibliothek

**Ein Titel fehlt.**
Liegt er in einem eingetragenen Musikordner? **Einstellungen → Neu einlesen**. Am Handy wird nur
der Ordner **Music** eingelesen.

**Bei vielen Titeln steht „Unbekannter Interpret“.**
Die Dateien haben keinen Interpreten-Tag. Amber Player versucht, ihn aus Dateinamen wie
„Interpret - Titel“ zu lesen. Am zuverlässigsten ist es, die Tags mit einem Tag-Editor (z. B.
Kid3, Picard) zu ergänzen und dann neu einzulesen.

**Zwei verschiedene Lieder heißen gleich – werden sie zusammengelegt?**
Nein, nie. Jede Datei bleibt ein eigener Titel.

**Ein Cover ist falsch oder fehlt.**
Amber Player übernimmt Internet-Cover nur, wenn Interpret und Titel wirklich passen. Ein eigenes
Cover legst du als `cover.jpg` oder `folder.jpg` in den Album-Ordner oder bettest es in die Datei
ein.

## Wiedergabe

**Es knackt.**
- Am PC: Effekte mit sehr starker Anhebung prüfen – der Schutz-Limiter fängt Spitzen zwar ab,
  aber weniger ist oft mehr.
- Am Handy: auf die neueste Version aktualisieren (größerer Audiopuffer seit 0.5.1).

**Die Musik am Handy stoppt im Hintergrund.**
Akkuoptimierung abschalten: Android-Einstellungen → Apps → Amber Player → Akku →
**Nicht eingeschränkt**.

**Der Equalizer ändert fast nichts.**
Alle Bänder gleich hoch angehoben ergibt nur „lauter“ – und das gleicht Amber Player aus, damit
nichts übersteuert. Heb gezielt einzelne Bereiche an oder nimm ein Preset.

**Amber Player läuft nach dem Schließen weiter.**
Nur solange Musik spielt. Ohne Musik beendet sich Amber Player komplett; nach dem Ende der Musik
oder einer Minute Pause ebenfalls.

## Handy und PC

**Das Handy findet den PC nicht.**
Gleiches WLAN? Amber Player am PC gestartet? **Im Heimnetz** eingeschaltet? Bei aktiver Firewall
die Freigabe bestätigt? Details: [Mit dem PC verbinden](Mit-dem-PC-verbinden.md).

**„Der PC hat die Verbindung abgelehnt“.**
Das Token wurde am PC neu erzeugt. Neu koppeln.

**Die App ist zweimal installiert (Samsung).**
Sie wurde zusätzlich im Profil „Dual Messenger“ installiert. Eine Kopie deinstallieren.

**Wie bekomme ich Musik vom PC aufs Handy, ohne sie zu kopieren?**
Mit der Kopplung: **Mehr → PC → Musik vom PC** oder eine Playlist mit **Hier** abspielen.

## Plugins

**Mein Klang-Plugin erscheint nicht.**
Seite **Effekte** öffnen – oben steht unter „Nicht geladen“, was nicht stimmt. Klang-Plugins
wirken erst nach einem Neustart der App.

**Meine Visualisierung zeigt nichts.**
Fehler erscheinen unten rechts als Meldung. Prüfe, ob `draw` vorhanden ist und die Datei auf
`.js` endet. Siehe [Visualisierung erstellen](Visualisierung-erstellen.md).

**Kann ein Plugin meinem Rechner schaden?**
Klang-Plugins nicht – sie enthalten keinen Programmcode. Visualisierungen laufen abgeschottet.
Quellen-Plugins sind Programme: nur aus vertrauenswürdigen Quellen installieren.

## Fernsteuerung

**Die API antwortet nicht.**
Läuft Amber Player? Standardadresse `http://127.0.0.1:7878`. Von einem anderen Gerät aus nur mit
**Im Heimnetz** und Token.

**Kann ich den Port ändern?**
Ja, unter **Einstellungen → Fernsteuerung → Port**. Gilt nach einem Neustart.
