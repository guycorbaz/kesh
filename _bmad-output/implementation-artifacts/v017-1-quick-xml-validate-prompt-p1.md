# Validation P1 de la fiche v017-1-quick-xml — prompt des deux lentilles (Sonnet, contexte frais)

Deux agents **Sonnet**, lancés en parallèle, chacun en contexte frais, chacun avec **une** lentille (R ou F). Langue :
français soigné. Worktree : `/home/gcorbaz/devel/kesh-0171` (branche `story/v017-1-quick-xml`). Fiche :
`_bmad-output/implementation-artifacts/v017-1-quick-xml.md` ; registre : `v017-1-quick-xml-choix-autonomes.md` ; issue
#789 (`gh issue view 789 --json title,body,comments`, lecture seule). Contrainte du Project Lead :
`/home/gcorbaz/devel/kesh-gate-logs/prompts/v017-donnees-a-garder.md` — et, pour l'import CAMT : **un relevé CAMT.053 déjà
importé en 0.16.x doit rester lisible et inchangé après la montée de quick-xml ; l'empreinte anti-doublon d'un fichier
ne doit pas changer.**

## Interdits — lecture seule stricte

Aucune commande qui écrit, compile ou exécute : ni `cargo` (sauf `cargo tree` en lecture), ni `npm`, ni `python3`, ni
`docker`, ni `curl`, ni `scripts/prepare-release.sh`, ni `scripts/test-fast.sh`, ni aucun script du dépôt ; aucun
`git checkout`, `switch`, `reset`, `rebase`, `stash`, `commit`, `push`, `tag` ; aucune commande `gh` qui écrit ; aucun
`rm`, `pkill`, `claude`. Aucune redirection (`>`, `>>`, `tee`), aucun fichier temporaire, rien dans le scratchpad : on
compare par des commandes enchaînées. Un fichier créé par erreur se déclare au rapport, il ne se supprime pas.
`~/kesh-test/` ne s'ouvre jamais. **La seule écriture permise est ton rapport**, par l'outil d'écriture de fichier, au
chemin ci-dessous.

Sources consultables en lecture : le worktree, `~/.cargo/registry/src/*/quick-xml-0.39.4/` (source 0.39.4),
`~/.cargo/advisory-db/crates/quick-xml/` (les deux avis), le Changelog amont
`https://github.com/tafia/quick-xml/blob/master/Changelog.md` et la source 0.42 sur GitHub (`tag v0.42.0`) par lecture
web si ton outillage le permet.

## Lentille R — réalité : chaque prémisse et chaque critère contre le code

Rapport : `/home/gcorbaz/devel/kesh-gate-logs/v017-1-quick-xml-validate-p1-R.md`.

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
5. **Mutations M1 à M6** : chacune fait-elle rougir le test annoncé ? En voyez-vous une qui survivrait sans le dire ?
6. **Manuel** : la fiche affirme qu'aucun texte visible ne change et qu'aucun manuel ne nomme `quick-xml` ; vérifier
   `docs/manual/fr/*.tex` **et** les PDF (`pdftotext <pdf> - | tr '\n' ' ' | tr -s ' ' | grep -ci "quick-xml\|RUSTSEC"`),
   et la clé i18n `bank-import-errors-malformed-xml` dans les quatre locales (le refus nouveau la réutilise-t-il bien ?).

## Lentille F — failles : ce que la fiche laisse passer

Rapport : `/home/gcorbaz/devel/kesh-gate-logs/v017-1-quick-xml-validate-p1-F.md`.

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
6. **Périmètre** : défauts voisins (entités, `spawn_blocking`) correctement tracés en issues demandées, sans
   élargissement ? Un défaut voisin manque-t-il ?

## Forme du rapport (les deux lentilles)

- Findings numérotés `R-n` / `F-n`, chacun avec sévérité (CRITICAL, HIGH, MEDIUM, LOW), la **ligne lue** (sortie
  `grep -nF` ou extrait copié) qui l'établit, et le remède proposé.
- **Liste des axes exercés et des axes non exercés** (avec la raison). Un « 0 finding » sans cette liste ne compte pas
  comme passe.
- Toute affirmation de présence ou d'absence s'appuie sur une sortie `grep -nF` copiée.
