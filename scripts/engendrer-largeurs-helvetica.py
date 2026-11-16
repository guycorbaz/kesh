#!/usr/bin/env python3
"""Engendre, ou vérifie, les deux tableaux de `crates/kesh-qrbill/src/largeurs.rs`
(Story 36-12a, #944 ; C-36-12-4, C-36-12a-2).

Usage, depuis la racine du dépôt :

    python3 -I scripts/engendrer-largeurs-helvetica.py            # écrit les tableaux sur la sortie standard
    python3 -I scripts/engendrer-largeurs-helvetica.py --verifier crates/kesh-qrbill/src/largeurs.rs

`--verifier` engendre les tableaux et les compare à ceux du fichier (lignes vides,
`#[rustfmt::skip]` et doc-comments `///` ôtés) : code 0 s'ils sont identiques, 1 sinon.
Le script n'écrit aucun fichier.

Sources :
- `lopdf` 0.31.0, du registre cargo : `src/encodings/mappings.rs` (`WIN_ANSI_ENCODING`,
  octet -> `Glyph::<nom>`) et `src/encodings/glyphnames.rs` (`<nom>` -> point de code) ;
- les métriques Adobe de 1990 d'Helvetica distribuées par TeX Live, paquet Debian/Ubuntu
  `texlive-fonts-recommended` (relevé en version 2025.20260124-1),
  `/usr/share/texlive/texmf-dist/fonts/afm/adobe/helvetic/phvr8a.afm` (Helvetica) et
  `phvb8a.afm` (Helvetica-Bold), lues par nom (`N <nom>`), leur colonne `C` étant
  l'AdobeStandardEncoding. Empreintes attendues dans `EMPREINTES` ; une autre empreinte est
  signalée sur la sortie d'erreur, sans arrêter le script.

Règles :
- octets 0x20 à 0xFF ; pour un caractère porté par plusieurs octets, le premier gagne, comme
  `lopdf::encodings::string_to_bytes` (`position`) ;
- `Euro`, absent des AFM de 1990, reçoit 556 en maigre et en gras (métriques Adobe
  actuelles d'Helvetica) ;
- un autre nom absent d'un AFM fait échouer le script.
"""
import glob
import hashlib
import os
import re
import sys

LOPDF = os.path.expanduser("~/.cargo/registry/src/*/lopdf-0.31.0/src/encodings")
AFM = "/usr/share/texlive/texmf-dist/fonts/afm/adobe/helvetic/"
EMPREINTES = {
    "phvr8a.afm": "2adf0a69189db6cf3d10120adbd5f62689835040f1beba1e689cff44d4e2eabe",
    "phvb8a.afm": "340c783381e32fbbc034225b73eb64d38564a25f93d28697790dd3da1f9752f3",
}
EURO = 556


def chasses(afm):
    with open(AFM + afm, "rb") as f:
        brut = f.read()
    empreinte = hashlib.sha256(brut).hexdigest()
    if empreinte != EMPREINTES[afm]:
        print(f"attention : {afm} a l'empreinte {empreinte}, non {EMPREINTES[afm]}",
              file=sys.stderr)
    table = {}
    for ligne in brut.decode("latin-1").splitlines():
        m = re.match(r"C -?\d+ ; WX (\d+) ; N (\S+) ;", ligne)
        if m:
            table[m.group(2)] = int(m.group(1))
    return table


def source_lopdf(nom):
    chemins = glob.glob(os.path.join(LOPDF, nom))
    if not chemins:
        sys.exit(f"lopdf 0.31.0 introuvable sous {LOPDF} (lancer un `cargo fetch`)")
    with open(chemins[0], encoding="utf-8") as f:
        return f.read()


def points_de_code():
    table = {}
    for ligne in source_lopdf("glyphnames.rs").splitlines():
        m = re.match(r"\s*pub const (\w+): u16 = 0x([0-9a-fA-F]+);", ligne)
        if m:
            table[m.group(1)] = int(m.group(2), 16)
    return table


def win_ansi():
    texte = source_lopdf("mappings.rs")
    bloc = texte.split("pub const WIN_ANSI_ENCODING: [Option<u16>; 256] = [", 1)[1]
    bloc = bloc.split("];", 1)[0]
    entrees = [e.strip() for e in bloc.split(",") if e.strip()]
    assert len(entrees) == 256, len(entrees)
    return [None if e == "None" else re.match(r"Some\(Glyph::(\w+)\)", e).group(1)
            for e in entrees]


def litteral(c):
    if c in ("'", "\\"):
        return "'\\" + c + "'"
    if ord(c) < 0x7F and c.isprintable():
        return "'" + c + "'"
    return "'\\u{%04X}'" % ord(c)


def tableaux():
    maigre, gras = chasses("phvr8a.afm"), chasses("phvb8a.afm")
    codes = points_de_code()
    vus, winansi, noms = set(), [], set()
    for octet, nom in enumerate(win_ansi()):
        if octet < 0x20 or nom is None:
            continue
        c = chr(codes[nom])
        if c in vus:
            continue
        vus.add(c)
        winansi.append((octet, c, nom))
        noms.add(nom)
    lignes = []
    for nom in sorted(noms):
        if nom == "Euro":
            assert nom not in maigre and nom not in gras
            r, b = EURO, EURO
        else:
            r, b = maigre[nom], gras[nom]
        lignes.append((nom, r, b))
    sortie = ["pub(crate) const WINANSI: [(u8, char, &str); %d] = [" % len(winansi)]
    sortie += ["    (0x%02X, %s, \"%s\")," % (o, litteral(c), n) for o, c, n in winansi]
    sortie += ["];", "", "pub(crate) const CHASSES: [(&str, u16, u16); %d] = [" % len(lignes)]
    sortie += ["    (\"%s\", %d, %d)," % ligne for ligne in lignes]
    sortie += ["];"]
    return sortie


def tableaux_du_fichier(chemin):
    """Les lignes des deux tableaux du fichier, du premier `const WINANSI` au `];` de `CHASSES`."""
    with open(chemin, encoding="utf-8") as f:
        lignes = f.read().splitlines()
    debut = next(i for i, l in enumerate(lignes) if l.startswith("pub(crate) const WINANSI"))
    fins = [i for i, l in enumerate(lignes) if l == "];" and i > debut]
    return lignes[debut:fins[1] + 1]


def utile(lignes):
    return [l for l in lignes
            if l.strip() and l.strip() != "#[rustfmt::skip]" and not l.lstrip().startswith("///")]


def main():
    engendre = tableaux()
    if len(sys.argv) == 3 and sys.argv[1] == "--verifier":
        attendu, lu = utile(engendre), utile(tableaux_du_fichier(sys.argv[2]))
        if attendu == lu:
            print(f"identiques : {len(lu)} lignes")
            return 0
        for i, (a, b) in enumerate(zip(attendu, lu)):
            if a != b:
                print(f"première différence, ligne utile {i + 1} :\n  engendré : {a}\n  fichier  : {b}")
                break
        else:
            print(f"longueurs différentes : {len(attendu)} engendrées, {len(lu)} lues")
        return 1
    if len(sys.argv) != 1:
        sys.exit(__doc__)
    print("\n".join(engendre))
    return 0


if __name__ == "__main__":
    sys.exit(main())
