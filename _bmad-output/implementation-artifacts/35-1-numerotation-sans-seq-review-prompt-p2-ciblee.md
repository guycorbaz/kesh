# Prompt — revue de code P2 ciblée de la Story 35-1 (#932, #948), une lentille, braquée sur la remédiation de la P1

Passe **ciblée**, une seule lentille, contexte frais, **Sonnet** (P1 : Opus, trois lentilles), en lecture seule. Worktree
`/home/gcorbaz/devel/kesh-e351`, branche `story/35-1-numerotation-sans-seq`. La P1 a rendu 3 MEDIUM et 16 LOW (rapports
`/home/gcorbaz/devel/kesh-gate-logs/e351-review-p1-{B,E,A}.md`). Déjà tracés, à ne pas rapporter de nouveau : #1114 (accolade dans le nom
d'exercice), #1115 (phrases `too_long` / `duplicate` qui présument leur cas, B4), #1116 (course entre deux administrateurs, E3), #1117
(test du format par défaut sur deux exercices de même année).

## Périmètre : le seul commit `84a6a0334`

`git -C /home/gcorbaz/devel/kesh-e351 show 84a6a0334` (18 fichiers). Compte rendu : entrée P1 du Change Log et annotations du Dev Agent
Record de `_bmad-output/implementation-artifacts/35-1-numerotation-sans-seq.md` (commit `bbb37bcaa`), choix C-e351-7 de
`e35-choix-autonomes-e351.md`. Journaux sous `/home/gcorbaz/devel/kesh-gate-logs/e351/` : `rem-p1-*.log`, `mut/campagne-rem-p1.log`.

Il remédie :
- **A1** — le code `NUMBER_FORMAT_UNRENDERABLE` cité au manuel utilisateur, PDF régénéré.
- **E1** — la borne de 64 du rendu reste **en octets** (C-e351-7 : la charge QR est bornée en octets, `kesh-qrbill/src/generator.rs:73`) ;
  le message (quatre langues), le manuel, `api-external.md`, le CHANGELOG et le doc-comment de `render` disent « 64 octets ; une lettre
  accentuée en compte deux ».
- **E2** — le miroir `invoice-number-format.ts` aligné sur `parse` (accolade isolée refusée, nouvelle clé
  `invoices-format-error-unmatched-brace` ; padding strictement décimal) ; un format mal formé chargé tel quel bloque le bouton.
- LOW : B1, B2 (garde de `into_number_duplicate` sur la clé en fin de message), B3/A3/E5 (compteurs assertés), B5/E6, A2, A4–A7, E4 (« un
  administrateur doit… », quatre langues, G9).

## Ce que tu vérifies

1. **C-e351-7** : la prémisse « le numéro entre dans la charge QR bornée en octets » est-elle vraie au code (lis `generator.rs` et le site
   qui y met le numéro) ? Une colonne `VARCHAR(64)` en `utf8mb4` accepte 64 **caractères** : la borne en octets est-elle donc
   seulement plus stricte, jamais plus lâche ? `worst_case_len` (validation du format) et le miroir TS comptent-ils de façon cohérente avec
   `render` (octets ou caractères — un format ASCII seul, donc égal ; mais `{FY}` compté 16 au pire : est-ce une borne en octets ou en
   caractères, et le nom d'exercice peut-il dépasser 16 octets) ? Le test `ac3_multibyte_fiscal_year_name_counts_bytes` rougirait-il si
   l'on passait à `chars().count()` ? Toutes les phrases « 64 caractères » ont-elles disparu
   (`git -C … grep -nE '64 (caractères|Zeichen|caratteri|characters)'`, quatre catalogues, manuels, `api-external.md`, CHANGELOG,
   commentaires) ?
2. **Le miroir** : compare ligne à ligne `invoice-number-format.ts` à `parse` / `validate_template_rules` de `invoice_format.rs`. Même
   ordre de contrôles, même code pour : `}` seule, `{` seule, `{}`, `{SEQ:}`, `{SEQ: 4}`, `{SEQ:4a}`, `{SEQ:+4}`, `{SEQ:04}`, `{SEQ:11}`,
   `{seq}`, `{YEAR}{YEAR}` ? `parseInt` restant ailleurs ? La nouvelle clé dans les **quatre** catalogues, repli égal au fr-CH,
   `sitesTotal` 3031 → 3032 au code (`i18n-keys.test.ts`), `lint-i18n-ownership` au journal. Les deux tests « chargé tel quel » désactivent-ils
   bien le bouton, et un format **valide** chargé reste-t-il enregistrable ?
3. **La garde B2** : `into_number_duplicate` lit la clé « en fin de message » (`for key '<clé>'` ou `.<clé>'`). Quel est le message exact
   de MariaDB 10.11 pour 1062 (le test en cite un : est-il plausible, avec ou sans préfixe de table) ? Une valeur de doublon qui contient
   `for key 'uq_invoices_number'` peut-elle tromper la garde (la valeur précède la clé) ? M9 est-elle tuée par le test nommé ?
4. **E4** : les quatre phrases nomment-elles l'administrateur, dans les quatre langues, repli Rust égal au catalogue (G9) ? Restent-elles
   vraies pour chaque cause (« aucun numéro n'a été consommé » : les compteurs assertés dans `duplicate` et `brace` — lis ces assertions ;
   M4c, M8b au journal) ?
5. **Le manuel** : `user-manual.tex` (l'item modifié) et le **PDF aplati** (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`
   puis `/usr/bin/grep -o`) : « NUMBER_FORMAT_ UNRENDERABLE » (coupé à l'`\allowbreak`), « 64 octets », « un administrateur ». Chaque
   phrase touchée vraie au code ?
6. **Les commentaires B1** : « clé d'API » retiré de quatre commentaires, dont deux antérieurs : les nouveaux disent-ils vrai (la route est
   réservée à l'administrateur, fermée aux clés : `lib.rs`) ?
7. **Attendus retouchés** : `git show 84a6a0334 -- '*.rs' '*.ts'`, lignes `-` dans les tests : un attendu existant a-t-il été affaibli
   sans motif ?
8. **Le compte rendu et les gates** au commit `84a6a0334` : test-fast 5391/5391 (13 ignorés), Vitest 3219 (233 fichiers), check, lint,
   build — chacun au journal, postérieur au commit ou déclaré « sources avant commit » ; rouges d'abord (`rem-p1-rouges-*.log`) ; `gate-ci`
   et E2E déclarés « à la clôture » (règle du 2026-11-13, `kesh-gate-logs/prompts/e35-dev-commun.md`), non affirmés ; l'entrée P1 du
   Change Log annotée sans réécriture des entrées antérieures (`git show bbb37bcaa`, lignes `-`).

## Interdits absolus

Lecture seule : aucun fichier écrit hormis ton rapport (outil Write), **aucune redirection** (`>`, `>>`, `tee`, `2>`, `2>&1`,
`2>/dev/null`, `> /dev/null` compris), **aucune substitution de processus** (`<(…)`), aucun fichier temporaire. Aucun `rm`, `pkill`, `kill`,
aucun `git` qui écrit. Aucun interpréteur ni script (`python3`, `perl`, `node`, `npm`, `npx`, `bash -c`, `sh -c`, `jq`, `scripts/*` dont
**`scripts/prepare-release.sh`**, les scripts de `kesh-gate-logs`, `make`, `latexmk`, `cargo`, `docker`, `mariadb`, `curl`, **`claude`**,
`sed -i`, `vitest`, `playwright`), aucune commande `gh`, aucun accès réseau. Ne lis rien sous `~/kesh/` ni `~/kesh-test/`. Permis :
`git show/diff/log/grep/ls-files`, `/usr/bin/grep`, `sed -n`, `awk` sans redirection ni `system()`, `cat`, `ls`, `find` sans
`-exec`/`-delete`, `wc`, `sort`, `uniq`, `cmp`, `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`. ⚠️ `grep` est une fonction vers `ugrep` :
emploie `/usr/bin/grep` ou `git grep`. Toute affirmation de présence ou d'absence s'appuie sur une sortie copiée au rapport. Un écart se
déclare.

## Rapport

`/home/gcorbaz/devel/kesh-gate-logs/e351-review-p2-ciblee.md` : findings numérotés (`C1`…), sévérité (CRITICAL/HIGH/MEDIUM/LOW), lieu,
ligne lue copiée, preuve, remède ; **axes exercés et non exercés** ; décompte final recompté depuis la liste.
