# Prompt — revue de code P1 de la story 49-5 (#1464, #1466), passe complète

Tu relis le **code** de la story de solde de l'Epic 49 de Kesh (comptabilité suisse ; Rust/Axum, MariaDB, SvelteKit), release
**0.20.0**. Passe **complète**, **contexte frais**, modèle **Opus** (première passe de la boucle ; la suivante changera de modèle).
Langue du rapport : français soigné. Aucune question : tu tranches et tu le dis. Kesh est **en production** (livres réels du
Project Lead, vraies factures émises) : rien de ce qui existe (factures, avoirs, écritures, PDF) ne doit changer.

## Cadre

- Worktree `/home/gcorbaz/devel/kesh-49-solde`, branche `story/49-5-solde-dette-epic`, partie de
  `origin/epic/49-types-de-lignes-facture` à `59fb28712`. Fiche (fait foi : D-A1 à D-B2, AC-A1 à AC-T2, tests T1 à T4, M1 à M3) :
  `_bmad-output/implementation-artifacts/49-5-solde-dette-epic.md`. Lot C (#1499) **retiré** : conception dans
  `49-5-lot-c-reporte-1499.md`, **hors revue**.
- **Diff de code à relire** : `git diff 59fb28712..HEAD -- frontend` (les commits `docs(49-5)` touchent des fiches, hors revue de
  code). Cinq fichiers de production ou de test : `invoices.types.ts`, `invoice-line-type.ts`, `invoice-lines.ts`,
  `invoice-line-request.types.test.ts` (neuf), `credit-notes/[id]/+page.svelte`, `credit-note-display-page.test.ts`.
- Ce qui a tourné : voir le Dev Agent Record de la fiche (journaux sous `/home/gcorbaz/devel/kesh-gate-logs/e49solde/` :
  `t1-rouge.log`, `m1.log`, `m2.log`, `t3-rouge.log`, `m3.log`, `front-*.log`, `gate-backend.log`, `e2e.log`).
- **Captures à regarder** (outil de lecture d'image), thème clair, 1280 px, base de démonstration :
  `/home/gcorbaz/devel/kesh-gate-logs/e49solde/captures/avoir-avant.png`, `avoir-avant-tableau.png`, `avoir-apres.png`,
  `avoir-apres-tableau.png`. Avant : « TVA %Compte de produit » et « 8.103000 — Ventes CI ». Après : « TVA % » séparé de
  « Compte de produit », « 8.10 » séparé de « 3000 — Ventes CI ». Compare aussi l'alignement du total (« 900.00 », « 950.00 »)
  avant/après et la ligne de commentaire.

## À lire d'abord

`CLAUDE.md` du worktree (§ *Qualité du code*, *Review Iteration Rule*, *Le prompt d'une passe doit NOMMER le manuel*, *Inventorier
les sites NON RÉSOLUS*, *Recette visuelle*), `docs/agents-orchestration.md` § 2, § 3, § 6, puis la fiche et le diff.

## Axes (chacun exercé, ou déclaré non exercé avec sa raison)

- **R1 — Lot A, le type** : l'union `CreateInvoiceLineRequest` est-elle exacte (quantité, prix, taux obligatoires pour `priced`,
  absents du texte) ? `revenueAccountId` reste optionnel ; les sites d'appel (`InvoiceForm.svelte:744-768`, `invoices.api.ts`)
  compilent sans changement ; aucun autre constructeur (`git grep CreateInvoiceLineRequest frontend`).
- **R2 — Le prédicat de type** : `estLigneDeTexte(t): t is InvoiceTextLineType` est-il honnête (valeur hors de l'union ; `null`
  et `undefined` : un `t` nul ne devient-il jamais « texte » à tort) ? Le rétrécissement dans les quatre gabarits/appelants
  (`InvoiceForm.svelte:1009`, `invoices/[id]/+page.svelte:1391`, `credit-notes/[id]/+page.svelte:182`, `invoice-lines.ts:53`)
  cache-t-il une branche `{:else}` rétrécie à `'priced'` que le code n'attendait pas ? `ligneAEnvoyer` : charge **identique**
  octet pour octet (tests `invoice-lines.test.ts`, `InvoiceForm.test.ts`) ; aucun `as`.
- **R3 — Les tests qui prouvent moins** : T1 (`@ts-expect-error` sur une ligne chacun) rougit-il vraiment sans le type (journal
  `t1-rouge.log` : cinq directives inutilisées) ? Une directive peut-elle être satisfaite par une autre erreur (un nom
  inexistant) ? M1 et M2 prouvent-elles ce qu'elles prétendent ? T3 : l'assertion `pr-2` est-elle vraie par construction ?
  Compte des cellules attendu (6, 6, 1, 2) juste ? M3 rougit-elle sur la seule cellule du taux ?
- **R4 — Lot B** : `pr-2` sur chaque `th`/`td` du tableau de `credit-notes/[id]/+page.svelte` ; le pied (`colspan="5"` et total)
  porte aussi `pr-2` (D-B1), contrairement au pied de la fiche de facture. Structure (`colspan`, nombre de cellules), textes, clés
  i18n et `data-testid` inchangés (D-B2) ; test T9 de la 49-1c passe inchangé.
- **R5 — Inventaire par le symptôme** : rejoue `git grep` de `quantity?:`, `unitPrice?:`, `vatRate?:` (frontend) et de
  `py-2 text-right">{line.vatRate}` ; cherche ce qui échappe (un autre type de requête où un champ exigé selon le type de ligne
  est optionnel ; une autre paire droite → gauche sans marge sur la fiche d'un avoir). Les sites hors module sont écrits comme
  issues I-2 à I-7 dans la fiche : confirme qu'aucun n'est resté sans classement.
- **R6 — Recette visuelle** : regarde les quatre captures ; le rendu montre-t-il ce que la fiche affirme ? Un défaut visible que
  les tests ne voient pas (total décalé, colonne trop serrée, débordement) ?
- **R7 — Le manuel, PDF compris** : la story ne touche **aucun** manuel (AC-T1). Vérifie-le (`git diff 59fb28712..HEAD --stat --
  docs`) et contre le code : aplatis les PDF des quatre langues, **sans redirection**, par
  `pdftotext /home/gcorbaz/devel/kesh-49-solde/docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -o '.\{80\}TVA %.\{80\}'`
  (et de même `avoir`, `Gutschrift`, `credit note`, `nota di credito`, `Compte de produit`) sur `docs/manual/{fr,de,en,it}/*.pdf`
  et dans les `.tex`. Une phrase décrit-elle la marge des colonnes de la fiche d'un avoir ou le type de la charge d'une ligne ?
- **R8 — Transverses** : aucun fichier de `crates/`, de migrations, des catalogues i18n, des manuels ni du PDF ne change ;
  commits `refs #N` sans mot-clé de fermeture : `git log 59fb28712..HEAD --format=%B | /usr/bin/grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'`
  (sortie vide attendue) ; chiffres du Dev Agent Record recomptés depuis les journaux.
- **R9 — Dev Agent Record** : chaque affirmation (gate, E2E, échecs jugés fichier par fichier, mutations) est-elle adossée à un
  journal ? Le story file n'affirme-t-il que ce qui a tourné ?

## Interdits

Lecture seule. N'écris **aucun** fichier du worktree ni du dépôt principal, hors ton rapport. Aucune commande qui écrit, compile
ou exécute : ni `cargo`, `npm`, `npx`, `node`, `python3`, `make`, `docker`, `mariadb`, `sqlx`, `flock`, ni **aucun** `scripts/*`
(en particulier **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/mem-guard.sh`,
`scripts/recette-montee.sh`). Ni `git commit`, `push`, `checkout`, `switch`, `reset`, `restore`, `stash`, `merge`, `rebase`,
`worktree`, `tag`. Aucun `gh` qui écrit. **Ni `pkill`, ni `killall`, ni `kill` par motif** ; aucun processus n'est à arrêter.
Aucune redirection (`>`, `>>`, `tee`), aucun fichier temporaire, aucun fichier dans le scratchpad. Permis : `git diff`, `git show`,
`git log`, `git grep`, `cat`, `sed -n`, `/usr/bin/grep` (`grep` est une fonction vers `ugrep`), `ls -l`, `pdftotext … -` en tube,
et l'outil de lecture d'image. Rien sous `~/kesh/`.

## Rapport

Écris **un seul fichier** : `/home/gcorbaz/devel/kesh-gate-logs/e49solde/revue-p1.md`, avec : le **verdict chiffré**
(CRITICAL / HIGH / MEDIUM / LOW) ; chaque finding numéroté `R1-n`, sévérité, prémisse **citée au code** (`chemin:ligne`, ligne
copiée, `grep -nF` copié pour toute affirmation d'absence ou de présence), remède proposé ; la liste des **axes exercés et non
exercés** (R1 à R9) avec ce qui a été lancé pour chacun — un « 0 finding » sans cette liste ne compte pas comme passe ; les
commandes lancées et leurs sorties utiles.
