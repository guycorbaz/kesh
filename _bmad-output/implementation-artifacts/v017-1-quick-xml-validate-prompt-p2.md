# Validation P2 de la fiche v017-1-quick-xml — prompt des deux lentilles (Opus, contexte frais, passe complète)

Deux agents **Opus**, lancés en parallèle, chacun en contexte frais, chacun avec **une** lentille (R ou F). Langue :
français soigné. Worktree : `/home/gcorbaz/devel/kesh-0171` (branche `story/v017-1-quick-xml`). Fiche :
`_bmad-output/implementation-artifacts/v017-1-quick-xml.md` ; registre : `v017-1-quick-xml-choix-autonomes.md` ; issue
#789 (`gh issue view 789 --json title,body,comments`, lecture seule). Contrainte du Project Lead :
`/home/gcorbaz/devel/kesh-gate-logs/prompts/v017-donnees-a-garder.md` — et, pour l'import CAMT : **un relevé CAMT.053 déjà
importé en 0.16.x doit rester lisible et inchangé après la montée de quick-xml ; l'empreinte anti-doublon d'un fichier
ne doit pas changer.** Décision de l'orchestrateur (P1, F-1) : un fichier **non UTF-8** hors du texte lu, accepté par
0.39 et refusé par 0.42, est une divergence **admise**, à condition d'être prouvée et écrite (C-v017-1-11).

C'est une passe **complète** (protocole entier sur toute la fiche), pas une passe ciblée. Les rapports de la P1 sont
à ta disposition (`/home/gcorbaz/devel/kesh-gate-logs/v017-1-quick-xml-validate-p1-{R,F}.md`) ; l'entrée « Validation
P1 » du Change Log de la fiche dit comment chaque finding a été traité. Ne les prends pas pour argent comptant.

## Interdits — lecture seule stricte

Aucune commande qui écrit, compile ou exécute : ni `cargo` (sauf `cargo tree` en lecture), ni `npm`, ni `python3`, ni
`docker`, ni `curl`, ni `scripts/prepare-release.sh`, ni `scripts/test-fast.sh`, ni aucun script du dépôt ; aucun
`git checkout`, `switch`, `reset`, `rebase`, `stash`, `commit`, `push`, `tag` ; aucune commande `gh` qui écrit ; aucun
`rm`, `pkill`, `claude`. Aucune redirection (`>`, `>>`, `tee`), aucun fichier temporaire, rien dans le scratchpad : on
compare par des commandes enchaînées. Un fichier créé par erreur se déclare au rapport, il ne se supprime pas.
`~/kesh-test/` ne s'ouvre jamais. **La seule écriture permise est ton rapport**, par l'outil d'écriture de fichier, au
chemin ci-dessous.

Sources consultables en lecture : le worktree, `~/.cargo/registry/src/*/quick-xml-0.39.4/` (source 0.39.4),
`~/.cargo/advisory-db/crates/quick-xml/` (les deux avis), **la source 0.42.0 extraite hors du dépôt**
(`/home/gcorbaz/devel/kesh-gate-logs/v017-1-src/quick-xml-0.42.0/`, à lire par `grep -n` et `sed -n`, sans rien y
exécuter ni compiler), et le Changelog amont `https://github.com/tafia/quick-xml/blob/master/Changelog.md`.

## Lentille R — réalité : chaque prémisse et chaque critère contre le code

Rapport : `/home/gcorbaz/devel/kesh-gate-logs/v017-1-quick-xml-validate-p2-R.md`.

Axes :

1. **Prémisses** de la section « Ce que le code dit » : chaque ligne citée (`mod.rs`, `bank_imports.rs`,
   `bank_import_fingerprint.rs`, `kesh-core/src/bank_imports.rs`, `pain001/mod.rs`, `Cargo.lock`, `ci.yml`,
   `quick-xml-0.39.4/src/name.rs:671`, `src/reader/mod.rs:289`) dit-elle ce que la fiche en dit ? Le défaut des
   références d'entité (`GeneralRef` ignoré, dernier fragment qui écrase) est-il réel dans 0.39.4 ?
2. **Inventaire des sites `quick-xml`** : est-il clos (`grep -rn "quick_xml" crates/`) ? Manque-t-il un site que 0.42
   casse (#963 : `AsRef<[u8]>` retiré, `decode()` retiré, `Cow<str>`, UTF-8 validé) ?
3. **Chaîne anti-doublon** : le hash de fichier, l'empreinte d'aperçu et la `DuplicateKey` de ligne dépendent-ils de ce
   que la fiche dit ? Existe-t-il un **autre** usage de la sortie du parseur qui persiste ou compare (rapprochement,
   `statement_id`, soldes, `details`) et qu'AC9 ne couvrirait pas ?
4. **Décidabilité des AC** : chaque AC (AC1 à AC14) se tranche-t-il par une commande ou un test nommé ? Les AC5 à AC7
   sont-ils vraiment rouges sous 0.39.4 (raisonner sur la source 0.39.4) et verts sous 0.42 (Changelog et, si possible,
   source 0.42) ? AC7 : la profondeur est-elle comptée sur tout élément ou seulement sur ceux qui déclarent ?
5. **Mutations M2 à M6** (M1 retirée en P1) : chacune fait-elle rougir le test annoncé ? En voyez-vous une qui
   survivrait sans le dire ?
6. **Manuel** : la fiche affirme qu'aucun texte visible ne change et qu'aucun manuel ne nomme `quick-xml` ; vérifier
   `docs/manual/fr/*.tex` **et** les PDF (`pdftotext <pdf> - | tr '\n' ' ' | tr -s ' ' | grep -ci "quick-xml\|RUSTSEC"`),
   et la clé i18n `bank-import-errors-malformed-xml` dans les quatre locales (le refus nouveau la réutilise-t-il bien ?).
7. **Le § *Établi dans la source 0.42.0*** : chaque ligne citée de la source 0.42.0 et de la source 0.39.4 dit-elle ce
   que la fiche en dit (`grep -nF` sur la ligne, copié) ?

## Lentille F — failles : ce que la fiche laisse passer

Rapport : `/home/gcorbaz/devel/kesh-gate-logs/v017-1-quick-xml-validate-p2-F.md`.

Axes :

1. **Données de la 0.16** : un relevé importé en 0.16 pourrait-il, après la montée, être lu autrement (date, montant,
   signe, référence, devise, IBAN, nom) sans qu'AC9 rougisse ? Les fixtures couvrent-elles les formes réelles des
   fichiers bancaires suisses (namespace préfixé, `xsi:schemaLocation`, BOM, CRLF, entités, CDATA, Latin-1, `DtTm`) ?
2. **Ordre des commits** : la fiche garantit-elle que les références sont produites **sous 0.39.4** et ne sont jamais
   régénérées après la montée ? Le mode `KESH_GOLDEN_REGEN` peut-il fausser cette garantie ?
3. **Tests hostiles** : faux vert possible (fixture qui n'atteint pas le chemin vulnérable — par exemple un `<Amt>` hors
   d'un `<Stmt>`, ou des attributs placés après `Ccy`) ? Instabilité en CI (debug, runners lents) ? Fil qui survit au
   délai ?
4. **Exposition oubliée** : d'autres lecteurs XML dans le dépôt (QR-facture, import de factures, sauvegarde) ? D'autres
   chemins de déni de service sur le même lecteur que 0.42 ne ferme pas ?
5. **Contrainte de la release** : aucune migration, aucun bump, aucun texte visible — la fiche le tient-elle ? Le choix
   0.42 plutôt que 0.41 (C-v017-1-1) est-il justifié, et ses risques (validation UTF-8 de toute l'entrée,
   normalisation des valeurs d'attribut, EOL) sont-ils couverts ?
6. **Périmètre** : défauts voisins (entités et CDATA → #819, `spawn_blocking` → #820) correctement tracés, sans
   élargissement ? Un défaut voisin manque-t-il ?
7. **Le verrou des références** (`KESH_GOLDEN_REGEN` gardé par la version résolue au `Cargo.lock` et par `CI`, contrôle
   `git diff --stat <sha-commit-A> HEAD` au § *Gates*) ferme-t-il réellement la régénération après la montée ? Par où
   passerait-on encore ?

## Axe obligatoire des deux lentilles — ce que la remédiation P1 vient d'écrire

Le motif mesuré du dépôt : *la sévérité se déplace vers ce qu'on vient d'écrire*. Lister le diff de la remédiation
(`git show --stat HEAD` puis `git show HEAD -- _bmad-output/implementation-artifacts/v017-1-quick-xml.md
_bmad-output/implementation-artifacts/v017-1-quick-xml-choix-autonomes.md`, commit `docs(v017-1) : remédiation de la
validation P1`) et relire **chaque passage ajouté ou modifié** : AC5, AC7, AC8, AC9 (fixtures, verrou, classes de
divergence), AC11, AC13, AC14, le tableau des mutations, le § *Gates*, le § *Établi dans la source 0.42.0*, le
§ *Ce que les gardes ne voient pas*, C-v017-1-1 (nuance), C-v017-1-4, C-v017-1-9, C-v017-1-11. Une contradiction
entre un passage neuf et un passage ancien, un chiffre non propagé, une cote fausse ou un critère devenu
indécidable est un finding. Le rapport consacre une section à cet axe.

## Forme du rapport (les deux lentilles)

- Findings numérotés `R-n` / `F-n`, chacun avec sévérité (CRITICAL, HIGH, MEDIUM, LOW), la **ligne lue** (sortie
  `grep -nF` ou extrait copié) qui l'établit, et le remède proposé.
- **Liste des axes exercés et des axes non exercés** (avec la raison). Un « 0 finding » sans cette liste ne compte pas
  comme passe.
- Toute affirmation de présence ou d'absence s'appuie sur une sortie `grep -nF` copiée.
