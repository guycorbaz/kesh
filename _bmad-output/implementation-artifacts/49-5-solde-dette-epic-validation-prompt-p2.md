# Prompt — validation P2 de la fiche 49-5 « Solde de la dette de l'Epic 49 » (#1464, #1466, #1499), passe complète

Tu valides la **fiche** de la story de solde de l'**Epic 49** de Kesh (comptabilité suisse ; Rust/Axum, MariaDB, SvelteKit), release
**0.20.0** (vendredi 2026-11-27, coupure le jeudi 2026-11-26 au soir ; la story doit être close avant le vendredi matin). Kesh tient
les livres réels du Project Lead, qui a déjà émis de vraies factures : rien de ce qui existe (factures, avoirs, écritures, PDF) ne
doit changer. **Aucune migration.** Deuxième passe : **complète**, **contexte frais**, modèle **Opus** (la P1 était Sonnet : 0/1/2/9, remédiée). Langue du rapport : français.

Worktree `/home/gcorbaz/devel/kesh-49-solde`, branche `story/49-5-solde-dette-epic`, partie de
`origin/epic/49-types-de-lignes-facture` à `59fb28712`. Fiche : `_bmad-output/implementation-artifacts/49-5-solde-dette-epic.md` (HEAD de la branche ; le code reste lu à `59fb28712`).
Rapport de la P1 : `/home/gcorbaz/devel/kesh-gate-logs/e49solde/validation-p1.md` ; la remédiation est le dernier commit de la
branche (`git log -1 --stat`, `git show HEAD`) et l'entrée « Validation P1 remédiée » du Change Log.
Pièces (lecture seule) : `/home/gcorbaz/devel/kesh-gate-logs/e49solde/issues-candidates.json`, `issues-recherche.json`,
`inv-1464.txt`, `inv-1466.txt` (régénéré par `outils/paires-droite-gauche.py`), `inv-1499.txt` ; capture `/home/gcorbaz/devel/kesh-gate-logs/e491c/captures/06-fiche-avoir.png`.

## À lire d'abord

1. `CLAUDE.md` du worktree : § *Priorités des défauts* (story de solde, plafond), *Numérotation des versions* (six stories par
   épique), *Qualité du code* (DRY), *Review Iteration Rule* et ses sous-sections (*Inventorier les sites NON RÉSOLUS*, *Le prompt
   d'une passe doit NOMMER le manuel*, *La passe ciblée* — définition d'un module), *Recette visuelle*, *Règle de splitting
   préventif*, *Plafonds mémoire*.
2. `docs/agents-orchestration.md` (§ 2, § 3, § 4).
3. `_bmad-output/planning-artifacts/epic-49-types-de-lignes-facture.md` (§ *Réduction du 2026-11-21*, § *Contraintes de
   production*, point 3 : réimpression à l'identique).
4. Les issues : d'abord dans `issues-candidates.json` (corps et commentaires). Si tu dois en relire une sur GitHub, **lecture
   seule** (`gh issue view N --json number,title,labels,body,comments,milestone`), et **une minute au moins entre deux requêtes
   `gh`** (attends par une commande qui ne fait rien d'autre ; jamais de rafale).
5. Les fiches sources : `49-1c-types-de-lignes-ecran.md` (constats B-1 et A-4 de la revue P1, D5 à D7), `49-1d-types-de-lignes-recette-montee.md`
   (origine de #1499), `49-1b-types-de-lignes-pdf.md`, `49-4a-capacite-pdf-avant-validation.md`, même dossier.
6. La fiche en entier, puis le rapport de la P1.

## Axes (chacun exercé, ou déclaré non exercé avec sa raison)

- **A1 — périmètre et plafond** : les trois issues sont-elles de la dette **des stories de l'Epic 49** (origine relue) ? Le plafond
  provisoire est-il écrit comme « à confirmer par le Project Lead » ? L'ordre de choix (PDF, puis P3, puis le reste) est-il tenu —
  en particulier : #1466 (P4) passe-t-elle à bon droit devant les P4 restantes, et chaque P3 écartée l'est-elle pour une raison
  vérifiable ? Le décompte des modules (deux) est-il juste au grain du `CLAUDE.md` (un domaine du frontend dont le **code de
  production** change ; une route `src/routes/(app)/credit-notes/` compte-t-elle comme domaine `credit-notes`) ?
- **A2 — prémisses relues au code** : chaque `chemin:ligne` cité est-il juste à `59fb28712` (ligne copiée) ? En particulier :
  `invoices.types.ts:23`, `:361-387`, `invoice-lines.ts:32`, `:84-96`, `invoice-line-type.ts:18-33`, `InvoiceForm.svelte:744-768`,
  `:1009-1024`, `credit-notes/[id]/+page.svelte:26`, `:166-216`, `invoices/[id]/+page.svelte:1374-1410`,
  `credit-note-display-page.test.ts:85-120`, `scripts/tests/recette-montee.test.sh:11`, `:19`, `:385-395`, `:472-481`, `:864-873`,
  `scripts/recette-montee.sh:153-157`, `:308-310`, `:317`, `crates/kesh-db/tests/recette_montee_script.rs`, `docs/testing.md:643`,
  `docs/ci.md:101`, et pour les écartées `crates/kesh-qrbill/src/pdf.rs:585`, `:624`, `:697`, `:706`, `:715`, `:735`,
  `crates/kesh-api/src/routes/invoices.rs:626`.
- **A3 — inventaire par le symptôme, ensemble clos** : rejoue les commandes en tête des trois `inv-*.txt` et cherche ce qu'elles
  **omettent** — lot A : un autre type de requête du frontend où un champ obligatoire selon le type de ligne est optionnel (avoirs,
  articles, factures fournisseur) ; un autre constructeur de charge de ligne ; lot B : une autre paire de cellules « droite puis
  gauche » sans marge (classes autres que `py-N`, composants partagés de tableau, fiches hors `routes/(app)`) ; lot C : une autre
  ressource partagée entre deux copies du test (fichiers fixes, ports, noms de conteneur factice, variables). Chaque site est-il
  corrigé ou écrit comme angle mort avec une raison vérifiable ?
- **A4 — décisions** : chaque D a-t-elle son option écartée ? D-A2 (prédicat de type) : le rétrécissement fonctionne-t-il vraiment
  sur `l.lineType` d'un objet dont le type est `LigneFormulaire` (propriété non optionnelle), et que deviennent les gabarits qui
  appellent `estLigneDeTexte` (branches `{:else}` rétrécies à `'priced'`) ? Le changement de sens annoncé pour une valeur hors de
  l'union est-il honnête ? D-B1 : `pr-2` sur la dernière colonne alignée à droite décale-t-il le total par rapport à la fiche de
  facture (comparer `invoices/[id]/+page.svelte`) ? D-C1 : un verrou `flock` par fichier tient-il si le descripteur est hérité par
  les processus de fond et par la relance sous environnement fermé de `recette-montee.sh` ; `PB + 1` est-il bien réservé ; la
  plage `23310…`/`23410…` avec un pas de 2 peut-elle faire se chevaucher `PA` d'une copie et `PB + 1` d'une autre ?
- **A5 — AC constatables** : chaque AC dit-il un test ou une commande, une valeur attendue, un fichier ? AC-A2 est-il tenable
  (vitest exécute aussi le fichier : les cas `@ts-expect-error` compilent-ils en JavaScript valide) ? AC-C3 est-il reproductible
  sous `scripts/mem-guard.sh` (deux exécutions à la fois sur toute la station) ?
- **A6 — preuve du rouge** : chaque test neuf rougit-il sans le correctif, ou par une mutation nommée avec son journal ? M2
  prouve-t-elle ce qu'elle prétend ? T3 est-il un test qui « vérifie qu'une classe existe » sans plus — et la recette visuelle
  compense-t-elle ? Si T6-rouge ne se reproduit pas, M4 suffit-il ?
- **A7 — factures réelles et données** : AC-T1 suffit-il à garantir qu'aucune facture, aucun avoir, aucune écriture, aucun PDF ne
  change ? Le lot A change-t-il une charge envoyée (comparer `ligneAEnvoyer` avant/après, `InvoiceForm.test.ts`) ?
- **A8 — manuel, PDF compris** : vérifie les manuels contre la fiche **et** contre le code, `.tex` et PDF des quatre langues
  (`ls docs/manual/*/` ; `pdftotext docs/manual/<l>/<f>.pdf - | tr '\n' ' ' | tr -s ' '` puis grep de `avoir`, `Gutschrift`,
  `credit note`, `nota di credito`, `Compte de produit`, `TVA %`, `recette`). Une phrase que la story rend fausse, ou une phrase
  à écrire (la fiche dit : aucune) ? `docs/testing.md` et `docs/ci.md` : la réécriture prévue par D-C3 et D-C4 est-elle juste et
  complète ?
- **A9 — écartées** (et #1504 désormais cité à `kesh-gate-logs/rel0200/verifs-0200.sh:15`) : pour #1399 et #1465 (écartées au code), la prémisse et la raison tiennent-elles ? Pour les autres, la raison
  est-elle vraie (milestone, hors dépôt, déjà corrigée — vérifie #1504 par `git log --oneline -S verifs-0200` ou la lecture du
  commit cité) ? Une issue née de l'épique manque-t-elle à la liste ?
- **A10 — charge et échéance** : l'estimation tient-elle avant le vendredi 2026-11-27 au matin, revue de code comprise ? La règle
  de splitting est-elle correctement jugée ?
- **A12 — chaque finding de la P1 réellement remédié** : pour V1-1 à V1-11, relis le finding, puis la fiche **et le code** :
  le remède est-il écrit, juste et complet, ou seulement annoncé au Change Log ? En particulier : V1-1 — D-C1 rend-elle le
  verrou réellement tenu par le shell principal (appel direct, `PORT_RESERVE`, sites `:394`, `:395`), `flock -n` et l'ouverture
  gardés sous `set -e`, et T5 (i) avec M5 rougissent-ils pour un verrou perdu dans un sous-shell (la sonde de fond rouvre-t-elle
  bien le fichier) ? V1-2 — le périmètre de #1499 et l'issue I-1 sont-ils écrits sans contradiction avec « une issue, une
  release » ? V1-3 — rejoue `python3 -I /home/gcorbaz/devel/kesh-gate-logs/e49solde/outils/paires-droite-gauche.py frontend/src`
  (lecture seule) : l'ensemble est-il clos et chaque site classé ; l'angle mort déclaré est-il juste ; le script omet-il une forme
  que tu trouves autrement ? V1-6/V1-7 — l'invocation de T6 et le dossier des verrous tiennent-ils ? La remédiation a-t-elle créé
  un défaut neuf (la sévérité se déplace vers ce qu'on vient d'écrire) ?
- **A13 — issues à ouvrir** : les textes I-1 à I-6 sont-ils exacts au code (`chemin:ligne` copiés), autonomes, d'une issue par
  problème, et de la bonne priorité selon le `CLAUDE.md` § *Priorités des défauts* ?
- **A11 — note de version, Dev Notes, commits** : numéros de release, jamais d'épique, dans la note de version ; `refs #N` ; aucun
  mot-clé de fermeture dans la fiche ni dans `git log origin/epic/49-types-de-lignes-facture..HEAD --format=%B`.

## Interdits

Lecture seule. N'écris **aucun** fichier du worktree ni du dépôt principal ; aucun `Edit`/`Write` hors de ton rapport. Aucun
`cargo`, `npm`, `npx`, `node`, `make`, `docker`, `mariadb`, `sqlx`, `flock`, `bash scripts/…`, aucun `scripts/*` (en particulier ni
`scripts/prepare-release.sh`, ni `scripts/test-fast.sh`, ni `scripts/gate-ci.sh`, ni `scripts/mem-guard.sh`, ni
`scripts/recette-montee.sh`, ni `scripts/tests/recette-montee.test.sh`). Aucun `git commit`, `git push`, `git checkout`,
`git switch`, `git stash`, `git merge`, `git reset`, `git worktree`, `git tag`. Aucun `gh` qui écrit (`issue
create/edit/comment/close`, `pr …`, `project …`, `api` avec `-X POST|PATCH|DELETE` ou une mutation GraphQL). Rien sous `~/kesh/`
(production). `pdftotext`, `git grep`, `git log`, `git show`, `git blame` et `python3 -I` en lecture (dont le script
`outils/paires-droite-gauche.py`, qui n'écrit rien) sont admis ; aucun autre script de `kesh-gate-logs/` (ni `verifs-0200.sh`, ni
`perte-assumee.sh`, ni `mut/*.py`) ne s'exécute.

## Rapport

Écris **un seul fichier** : `/home/gcorbaz/devel/kesh-gate-logs/e49solde/validation-p2.md`, avec :

- le **verdict chiffré** : CRITICAL / HIGH / MEDIUM / LOW ;
- chaque finding numéroté `V2-n`, sévérité, lot (A, B, C, écartées ou transverse), prémisse **citée au code** (`chemin:ligne`, ligne
  copiée), remède proposé ;
- la liste des **axes exercés** et **non exercés** (A1 à A13), avec ce qui a été lancé pour chacun ; un « 0 finding » sans cette
  liste ne compte pas comme passe ;
- les commandes lancées et leurs sorties utiles.
