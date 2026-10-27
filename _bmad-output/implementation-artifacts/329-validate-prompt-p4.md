# Prompt — validation P4, Story 32-9 : passe ciblée « regression hunter »

*Versionné le 2026-10-25. Passe **ciblée** (`CLAUDE.md`, § *La passe ciblée*) : **une seule lentille**, en contexte
frais, **Opus**, braquée sur **le seul commit de la remédiation de la P3**. Lecture seule stricte, **sans `gh`**, **sans
interpréteur** (`awk` compris), **sans redirection**.*

## Le périmètre, figé par les SHA

- Dépôt : `/home/gcorbaz/devel/kesh-32-9`, branche `story/32-9`.
- **Le commit à relire** : `09367214` (« remédiation de la validation P3 »), sur son parent `e2bea882`. Commande :
  `git -C /home/gcorbaz/devel/kesh-32-9 diff e2bea882 09367214 -- _bmad-output/implementation-artifacts/`.
- Fichiers : `_bmad-output/implementation-artifacts/32-9-outils-de-lecture.md` (la fiche),
  `epic-32-choix-autonomes-329.md` (le registre), `sprint-status.yaml` (entrées `32-9a-*`, `32-9b-*`).
- Ce que la remédiation a corrigé, et pourquoi : l'entrée « validation P3 » du Change Log de la fiche, et les rapports
  `/home/gcorbaz/devel/kesh-gate-logs/329-validate-p3-R.md`, `/home/gcorbaz/devel/kesh-gate-logs/329-validate-p3-F.md`.
- Le contexte que la remédiation invoque, à lire **seulement là où le diff y renvoie** : la fiche 32-8
  (`/home/gcorbaz/devel/kesh-32-8/_bmad-output/implementation-artifacts/32-8-validation-avant-envoi.md`, sa note
  l. 1538-1545 au commit `ef7756af`), la fiche 32-6 (`/home/gcorbaz/devel/kesh-32-6/…/32-6-endpoint-mcp.md` à
  `ecb26dc8`, pour les numéros de ligne décalés), le code du dépôt à `fa5b3468` (`crates/`, inchangé sur la branche).

**Le motif mesuré qui justifie la passe** : sur cette fiche, la P2 a trouvé un HIGH écrit par la remédiation de la P1,
et la P3 un MEDIUM écrit par celle de la P2. *La sévérité se déplace vers ce qu'on vient d'écrire* : ce qui reste à
relire, c'est ce commit-ci.

## La lentille — regression hunter

Pour **chaque hunk** du diff, réponds à trois questions, preuve à l'appui (commande et sortie copiée) :

1. **Le correctif est-il juste à sa source ?** Relis le site cité (`sed -n`, `grep -nF`) : le code, la migration, la
   fiche voisine disent-ils ce que le hunk affirme ? En particulier :
   - le § *Les jeux d'arguments de l'inventaire* : chaque jeu fait-il **réellement** sortir les lignes qu'il nomme —
     un compte `Asset` non bancaire est-il lettrable (`kesh-db/src/repositories/letterings.rs:363-365`) et au bilan
     (`kesh-report/src/balance_sheet.rs:296-301`) ; un compte `Expense` au compte de résultat (`income_statement.rs:76-81`) ;
     l'écriture E (K2 au débit, K1 au crédit) rend-elle les deux soldes non nuls, le poste ouvert de K1, les contreparties
     croisées au grand livre (`general_ledger.rs`, `fetch_counterparts`) ; le compte `Payable` porte-t-il le libellé de
     l'écriture d'achat et le numéro de la facture fournisseur comme pièce d'un poste ouvert (`open_items.rs`,
     `DocumentOwner`) ? Une ligne à piège de l'inventaire est-elle **sans** jeu ?
   - la tranche de `kesh_balance_agee` : `rapports.rs:661-665`, `aged_receivables.rs:77-93` ; la taille mesurée
     (287 / 279 octets) et les bornes 40 / 75 tiennent-elles sous 24 576 octets avec l'en-tête ?
   - la note pour la 32-8 : égale-t-elle, en substance, la note de la fiche 32-8 (l. 1538-1545) ; le grep prescrit
     (`grep -nE '65 ?536|65 ?537|64 ?Kio'`) attrape-t-il toutes les graphies présentes dans la fiche 32-8 (exécute-le) ;
     la règle « frontend relevé et déclaré, non amendé » garde-t-elle la 32-9a à cinq modules ?
   - `reconciliation.rs:4130`, `accounts.rs:224`, la contrainte `:36-50`, les lignes de la fiche 32-6 décalées de + 7
     (vérifie-en trois au moins à `ecb26dc8`), `kesh-api/Cargo.toml:84` (`regex`) ;
   - le CHANGELOG compté avec le manuel : l'écart à la convention est-il déclaré **partout** où le compte des modules
     s'écrit (fiche, § *Le découpage* et son tableau ; registre C-32-9-1 ; `sprint-status.yaml`) ?
2. **Le correctif introduit-il un défaut neuf ?** Une contradiction avec une autre section de la fiche ou du registre
   (une phrase restée à l'ancienne valeur, une mutation renumérotée sans son test, un test renommé sans ses renvois, un
   décompte qui ne suit plus), un test qui passe à vide, une mutation qui n'est plus tuée par le test nommé (Ma2, Ma28,
   Ma30, Mb3 en particulier), une prémisse fausse.
3. **Le symptôme corrigé reste-t-il ailleurs ?** Greppe la **valeur**, non la formulation (`CLAUDE.md`, § *Propagation
   post-patch*) : `Touchés dans la fiche 32-8`, `Ma1 à Ma2[0-9]`, `accounts.rs:223`, `36-42`, `un compte au numéro`,
   `balance_agee` + `— | — |`, `description(: `, `l\. (421|477|495|527|550|571|662)\b` sur la fiche **et** le registre ;
   et recompte, à la source : 31 AC (24 / 7), 16 tâches (10 / 6), 39 lignes de mutation (32 / 7), C-32-9-1 à 32.

**Axes obligatoires** (`CLAUDE.md`) : (A) **l'inventaire clos** — le nouveau § *Les jeux d'arguments* couvre-t-il toutes
les lignes à piège, et seulement des lignes qui existent ? (B) **le manuel** — les phrases que le diff ajoute ou change à
l'AC-a23 (balance âgée, « notamment », CHANGELOG) sont-elles vraies contre le code, et le PDF aplati
(`pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '`) ne contient-il encore rien qu'elles
contrediraient ?

Hors périmètre : tout ce que le diff ne touche pas — sauf une contradiction qu'il crée avec une section intacte.

## Ce que tu rends

Findings `H1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, hunk ou ligne de la fiche, site du dépôt ou de la fiche
voisine, preuve (commande et sortie copiée), correction proposée ; pour chacun : **le défaut est-il né du commit
`09367214`** ou lui est-il antérieur ? Un bilan d'en-tête **cohérent avec le corps**. ⛔ **La liste des axes exercés ET
non exercés** (les trois questions, pour chaque hunk ou groupe de hunks, et les axes A et B) — sans elle, « 0 finding »
ne compte pas comme passe. Rapport écrit **d'un seul jet**, avec ton outil d'écriture de fichier (jamais par une
redirection du shell), dans `/home/gcorbaz/devel/kesh-gate-logs/329-validate-p4.md`. Dernier message : chemin, bilan
par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport, **aucun fichier temporaire**, rien dans le scratchpad ; **aucune
redirection** (`>`, `>>`, `2>`, `2>&1`, `tee`, `<<`), pas même vers un fichier jetable ni vers `/dev/null` — un fichier
créé par erreur se déclare au rapport, il ne se supprime pas. Aucun `git checkout`/`switch` sous aucune forme ; aucune
commande qui écrit, compile ou exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, **`scripts/test-fast.sh`**,
**`scripts/mem-guard.sh`** et **`scripts/regen-test-schema.sh`**), `make`, `latexmk`, `cargo`, `rustup`, `npm`, `npx`,
`docker`, `curl`, `wget`, **`python3`, `python`, `perl`, `ruby`, `node`, `awk`** — `awk` **compris, même en simple filtre
d'un tube, même sans programme** (les lentilles de la P2 et de la P3 l'ont employé : c'est un écart à déclarer, non une
tolérance) — et tout autre interpréteur, `mariadb`/`mysql`, `sed -i`, `git commit|add|stash|fetch|pull|reset|rebase|tag|push`,
et **`gh` sous toutes ses formes** (même en lecture). `pdftotext` vers la sortie standard seulement. Aucune donnée réelle
de Guy (rien de `~/kesh-test/`). Le dépôt principal `/home/gcorbaz/devel/kesh` est hors de ton périmètre ; les worktrees
`kesh-32-6`, `kesh-32-8`, `kesh-32-10a` se lisent, ne s'écrivent pas. Autorisés : lecture, `grep`, `sed -n`,
`git log|show|diff|blame|merge-base`, `pdftotext … -`, et les filtres en lecture seule `tr`, `sort`, `cut`, `head`,
`tail`, `wc`, `uniq`, `printf` vers la sortie standard (`printf … | wc -c` et `wc -m` pour mesurer). Si une commande
t'est refusée, arrête-toi et dis-le dans ton rapport.
