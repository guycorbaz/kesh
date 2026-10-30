# Validation P3 de la fiche v017-1-quick-xml — passe ciblée (Sonnet, une lentille, contexte frais)

Un agent **Sonnet**, en contexte frais, **une seule lentille**. Langue : français soigné. Worktree :
`/home/gcorbaz/devel/kesh-0171` (branche `story/v017-1-quick-xml`). Fiche :
`_bmad-output/implementation-artifacts/v017-1-quick-xml.md` ; registre : `v017-1-quick-xml-choix-autonomes.md`.

C'est une **passe ciblée** (`CLAUDE.md` § *La passe ciblée*) : elle est braquée sur **le seul commit de la remédiation
P2**, `docs(v017-1) : remédiation de la validation P2 (refs #789)` (le dernier commit de la branche,
`git log -1 --format=%H`). Les rapports P2 qui l'ont motivée :
`/home/gcorbaz/devel/kesh-gate-logs/v017-1-quick-xml-validate-p2-{R,F}.md` ; l'entrée « Validation P2 » du Change Log
de la fiche dit comment chacun a été traité. Contrainte du Project Lead : un relevé CAMT.053 déjà importé en 0.16.x
reste lisible et inchangé après la montée ; l'empreinte anti-doublon d'un fichier ne change pas ; **une seule**
divergence admise, le refus d'un fichier non UTF-8 hors du texte lu (C-v017-1-11).

Rapport : `/home/gcorbaz/devel/kesh-gate-logs/v017-1-quick-xml-validate-p3.md`.

## Interdits — lecture seule stricte

Aucune commande qui écrit, compile ou exécute : ni `cargo` (sauf `cargo tree` en lecture), ni `npm`, ni `python3`, ni
`docker`, ni `curl`, ni `iconv`, ni `scripts/prepare-release.sh`, ni `scripts/test-fast.sh`, ni aucun script du dépôt ;
aucun `git checkout`, `switch`, `reset`, `rebase`, `stash`, `commit`, `push`, `tag` ; aucune commande `gh` qui écrit ;
aucun `rm`, `pkill`, `claude`. Aucune redirection (`>`, `>>`, `tee`), aucun fichier temporaire, rien dans le
scratchpad. Un fichier créé par erreur se déclare au rapport, il ne se supprime pas. `~/kesh-test/` ne s'ouvre jamais.
**La seule écriture permise est ton rapport**, par l'outil d'écriture de fichier, au chemin ci-dessus.

Sources en lecture : le worktree ; `~/.cargo/registry/src/*/quick-xml-0.39.4/` ; la source 0.42.0 extraite
`/home/gcorbaz/devel/kesh-gate-logs/v017-1-src/quick-xml-0.42.0/` (`grep -n`, `sed -n` seulement).

## Axes

1. **Le diff de la remédiation P2** (`git show HEAD -- _bmad-output/implementation-artifacts/v017-1-quick-xml.md
   _bmad-output/implementation-artifacts/v017-1-quick-xml-choix-autonomes.md`) : chaque passage ajouté ou modifié est-il
   juste au code et à la source amont (cotes relues par `grep -nF`, sorties copiées) ? Introduit-il une contradiction
   avec un passage non touché de la fiche ? En particulier :
   - l'adaptation `quick_xml::escape::unescape(&attr.value)` reproduit-elle **exactement** `unescape_value` de 0.39.4
     (décodage, entités prédéfinies, références de caractère, absence de normalisation) ?
   - l'exception d'UTF-8 conditionnée par la version lue au `Cargo.lock`, écrite dès le commit A : le test est-il vert
     sous 0.39 au commit A **et** sous 0.42 au commit B sans qu'aucun fichier de la caractérisation ne bouge ?
   - le garde d'inventaire des messages `MalformedXml` : l'inventaire des sites (`grep -n "MalformedXml"
     crates/kesh-import/src/camt053/mod.rs`) est-il exact et clos ? Le garde est-il décidable tel qu'écrit (que compte-t-il
     exactement, et le commit B — qui retire des relais `e.to_string()` — le fait-il rougir à tort) ?
   - la fixture `v04_characterization.xml` à deux `<Ntry>` : les marqueurs prescrits atteignent-ils bien la sortie
     (`mod.rs:557-650`, replis `TxDtls` → `Ntry`) ? Le `Ccy` à tabulation passe-t-il le chemin de `TxDtls/Amt`
     (`mod.rs:168-190`) ?
   - le contrôle des octets (AC9) : chaque commande est-elle juste et décidable (BOM, CR, tabulation, `0xE9`, `iconv`,
     motif ASCII de `v04_latin1_declare_ascii.xml`) ?
2. **Propagation** : chaque correction P2 est-elle reportée partout où elle doit l'être ?
   - **registre** : C-v017-1-3 (règle des messages, cinq fixtures), C-v017-1-4 (témoin, relevé complet), C-v017-1-9
     (emplacement du test pain.001), C-v017-1-10 (contrôle réel à Guy), C-v017-1-11 (une seule divergence, ordre
     parse/hash) ;
   - **mutations** : M4, M4-bis, M7 — chacune fait-elle rougir le test annoncé, et la variante `XmlVersion` de M4-bis
     existe-t-elle (`src/lib.rs` de 0.42) ?
   - **§ *Gates*** : les trois commandes du contrôle des références sont-elles justes (syntaxe git, chemins existants
     ou à créer, motifs entre guillemets) et couvrent-elles tout ce que la fiche dit ne pas changer après le commit A ?
     Le contrôle sur relevé réel est-il bien confié à Guy, après la sauvegarde ?
   - **CHANGELOG (AC14)** : la phrase prescrite ne mentionne-t-elle qu'**une** exception, cohérente avec AC9 et
     C-v017-1-11 ? Ne promet-elle rien que les angles morts démentent (chemin `RltdPties`, SipHash, #820) ?
   - **AC10/AC12 et Tâches** : AC10 au commit C, AC12 en test d'intégration ; la tâche 1 et les Dev Notes le disent-elles
     sans reste de l'ancienne rédaction (« commit d'AC9 », « `src` » pour pain.001) ?
3. **Valeurs** : greper les **jetons** changés par la remédiation (`grep -n "\bquatre\b\|\bcinq\b\|\b10 lignes\b\|:1036\|:1081\|:1237\|:1249"`)
   et signaler toute occurrence vivante restée à l'ancienne valeur. Les entrées passées du Change Log (Fiche créée,
   Validation P1) sont des relevés datés : n'y signale rien qui était juste à leur date.

## Forme du rapport

- Findings numérotés `P3-n`, sévérité (CRITICAL, HIGH, MEDIUM, LOW), **ligne lue** (sortie `grep -nF` ou extrait
  copié), remède proposé.
- **Liste des axes exercés et des axes non exercés** (avec la raison). Un « 0 finding » sans cette liste ne compte pas
  comme passe.
- Toute affirmation de présence ou d'absence s'appuie sur une sortie `grep -nF` copiée.
