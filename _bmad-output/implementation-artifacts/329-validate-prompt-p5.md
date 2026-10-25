# Prompt — validation P5, Story 32-9 : passe ciblée « regression hunter »

*Versionné le 2026-10-25. Passe **ciblée** (`CLAUDE.md`, § *La passe ciblée*) : **une seule lentille**, en contexte
frais, **Sonnet**, braquée sur **le seul commit de la remédiation de la P4**. Lecture seule stricte, **sans `gh`**, **sans
interpréteur** (`awk` compris), **sans redirection**.*

## Le périmètre, figé par les SHA

- Dépôt : `/home/gcorbaz/devel/kesh-32-9`, branche `story/32-9`.
- **Le commit à relire** : `e9dc5af7` (« remédiation de la validation P4 »), sur son parent `fdf37bcd`. Commande :
  `git -C /home/gcorbaz/devel/kesh-32-9 diff fdf37bcd e9dc5af7 -- _bmad-output/implementation-artifacts/`.
- Fichiers : `_bmad-output/implementation-artifacts/32-9-outils-de-lecture.md` (la fiche),
  `epic-32-choix-autonomes-329.md` (le registre), `sprint-status.yaml` (entrées `32-9a-*`, `32-9b-*`).
- Ce que la remédiation a corrigé, et pourquoi : l'entrée « validation P4 » du Change Log de la fiche, et le rapport
  `/home/gcorbaz/devel/kesh-gate-logs/329-validate-p4.md`.
- Le contexte que la remédiation invoque, à lire **seulement là où le diff y renvoie** : la fiche 32-8
  (`/home/gcorbaz/devel/kesh-32-8/_bmad-output/implementation-artifacts/32-8-validation-avant-envoi.md`, sa note
  l. 1538-1546 au commit `8747944d`), la fiche 32-6 (`/home/gcorbaz/devel/kesh-32-6/…/32-6-endpoint-mcp.md` à
  `5f6733be`, l'état de référence unique désormais), le code du dépôt à `fa5b3468` (`crates/`, inchangé sur la branche).

**Le motif mesuré qui justifie la passe** : sur cette fiche, la P2 a trouvé un HIGH écrit par la remédiation de la P1,
la P3 un MEDIUM écrit par celle de la P2, et la P4 trois MEDIUM écrits par celle de la P3. *La sévérité se déplace vers
ce qu'on vient d'écrire* : ce qui reste à relire, c'est ce commit-ci. La remédiation a choisi, à dessein, la forme la plus
simple (retrait, citation plutôt que recopie) : vérifie qu'elle n'a rien laissé de l'ancienne forme.

## La lentille — regression hunter

Pour **chaque hunk** du diff, réponds à trois questions, preuve à l'appui (commande et sortie copiée) :

1. **Le correctif est-il juste à sa source ?** Relis le site cité (`sed -n`, `grep -nF`). En particulier :
   - **H1** — les numéros `0P329{M}{nn}` de K1 et K2 : `ORDER BY number` (`crates/kesh-db/src/repositories/accounts.rs:391`,
     `:399`) et `ORDER BY a.number ASC` (`crates/kesh-report/src/trial_balance.rs:173`) ; la collation de la colonne
     `accounts.number` (`crates/kesh-db/migrations/20260411000001_accounts.sql`) range-t-elle `0P…` avant `1000` ? aucun
     plan de `crates/kesh-core/assets/charts/*.json` n'a-t-il un numéro en `0` (`grep -c '"number": "0'`) ? `MOTIF_PIEGE`
     reconnaît-il `P329A01` dans `0P329A01` (`printf '%s\n' '0P329A01' | grep -oE 'P(IEGE_329_|329)[AB][0-9]{2}'`) ?
     8 caractères tiennent-ils dans `VARCHAR(10)`, et la règle des formes (§ *Les pièges et la largeur des colonnes*)
     reste-t-elle cohérente avec cette exception ?
   - **H2** — la note pour la 32-8 cite-t-elle fidèlement la note de la fiche 32-8 **au commit `8747944d`**
     (`git -C /home/gcorbaz/devel/kesh-32-8 show 8747944d:_bmad-output/implementation-artifacts/32-8-validation-avant-envoi.md`,
     l. 1538-1546) : motif, valeur dérivée `65 530` → `24 570`, sites ? Exécute le motif sur cette fiche : attrape-t-il
     `65_536` ?
   - **H3** — la règle « manuel et CHANGELOG amendés par la 32-9a, frontend et `kesh-i18n` déclarés » tient-elle à cinq
     modules avec la convention écrite (§ *Le découpage*, l. 186-188, et son tableau) ?
   - **L1** — l'en-tête fixe `5f6733be` pour la fiche 32-6 : vérifie **au moins cinq** des lignes citées de cette fiche
     à cet état (`git -C /home/gcorbaz/devel/kesh-32-6 show 5f6733be:_bmad-output/implementation-artifacts/32-6-endpoint-mcp.md`),
     dont `l. 523` et `l. 717` ;
   - **L2 à L7** — `CHANGELOG.md` l. 11 et ses sections ; `journal_entries.rs:2165-2179` (`invoice_number` posé pour un
     règlement) ; le jeu J2 de `kesh_pieces` ; la liste de couples (outil, jeu) de l'AC-a17 ; Ma28a et Ma28b et la
     fixture de plus de 75 clients débiteurs.
2. **Le correctif introduit-il un défaut neuf ?** Une contradiction avec une autre section de la fiche ou du registre
   (une phrase restée à l'ancienne valeur, une mutation renumérotée sans son test, un test renommé sans ses renvois, un
   décompte qui ne suit plus), un test qui passe à vide, une mutation qui n'est plus tuée par le test nommé (Ma27, Ma28a,
   Ma28b, Ma30 en particulier), une prémisse fausse.
3. **Le symptôme corrigé reste-t-il ailleurs ?** Greppe la **valeur**, non la formulation (`CLAUDE.md`, § *Propagation
   post-patch*) : `65 ?536|65 ?537`, `| Ma28 |`, `P329A01\` est ASCII`, `sept octets`, `nomme \*\*son\*\* jeu`,
   `soumis à l'orchestrateur`, `à trancher par l'orchestrateur`, `qu'un \`### Modifié\``, `l\. 10,`, `ecb26dc8`,
   `l\. (516|710)\b` sur la fiche **et** le registre ; et recompte, à la source : 31 AC (24 / 7), 16 tâches (10 / 6),
   40 lignes de mutation (33 / 7), C-32-9-1 à 32.

**Axes obligatoires** (`CLAUDE.md`) : (A) **l'inventaire clos** — avec les numéros préfixés, chaque ligne à piège a-t-elle
encore un couple (outil, jeu) qui la fait sortir, et aucun jeu ne nomme-t-il une ligne absente ? (B) **le manuel** — les
phrases que le diff ajoute ou change (CHANGELOG, note pour la 32-8, amendement du manuel par la 32-9a) sont-elles vraies
contre le code et contre la fiche 32-8, et le PDF aplati (`pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' |
tr -s ' '`) ne contient-il rien qu'elles contrediraient ?

Hors périmètre : tout ce que le diff ne touche pas — sauf une contradiction qu'il crée avec une section intacte.

## Ce que tu rends

Findings `H1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, hunk ou ligne de la fiche, site du dépôt ou de la fiche
voisine, preuve (commande et sortie copiée), correction proposée ; pour chacun : **le défaut est-il né du commit
`e9dc5af7`** ou lui est-il antérieur ? Un bilan d'en-tête **cohérent avec le corps**. ⛔ **La liste des axes exercés ET
non exercés** (les trois questions, pour chaque hunk ou groupe de hunks, et les axes A et B) — sans elle, « 0 finding »
ne compte pas comme passe. Rapport écrit **d'un seul jet**, avec ton outil d'écriture de fichier (jamais par une
redirection du shell), dans `/home/gcorbaz/devel/kesh-gate-logs/329-validate-p5.md`. Dernier message : chemin, bilan
par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport, **aucun fichier temporaire**, rien dans le scratchpad ; **aucune
redirection** (`>`, `>>`, `2>`, `2>&1`, `tee`, `<<`), pas même vers un fichier jetable ni vers `/dev/null` — un fichier
créé par erreur se déclare au rapport, il ne se supprime pas. Aucun `git checkout`/`switch` sous aucune forme ; aucune
commande qui écrit, compile ou exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, **`scripts/test-fast.sh`**,
**`scripts/mem-guard.sh`** et **`scripts/regen-test-schema.sh`**), `make`, `latexmk`, `cargo`, `rustup`, `npm`, `npx`,
`docker`, `curl`, `wget`, **`python3`, `python`, `perl`, `ruby`, `node`, `awk`** — `awk` **compris, même en simple filtre
d'un tube, même sans programme** (les lentilles de la P2, de la P3 l'ont employé, et la P4 a vu le harnais ranger une sortie dans un fichier : c'est un écart à déclarer, non une
tolérance) — et tout autre interpréteur, `mariadb`/`mysql`, `sed -i`, `git commit|add|stash|fetch|pull|reset|rebase|tag|push`,
et **`gh` sous toutes ses formes** (même en lecture). `pdftotext` vers la sortie standard seulement. Aucune donnée réelle
de Guy (rien de `~/kesh-test/`). Le dépôt principal `/home/gcorbaz/devel/kesh` est hors de ton périmètre ; les worktrees
`kesh-32-6`, `kesh-32-8`, `kesh-32-10a` se lisent, ne s'écrivent pas. Autorisés : lecture, `grep`, `sed -n`,
`git log|show|diff|blame|merge-base`, `pdftotext … -`, et les filtres en lecture seule `tr`, `sort`, `cut`, `head`,
`tail`, `wc`, `uniq`, `printf` vers la sortie standard (`printf … | wc -c` et `wc -m` pour mesurer). Si une commande
t'est refusée, arrête-toi et dis-le dans ton rapport.
