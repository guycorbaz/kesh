# Revue de code P2 — 34-4 « transaction déjà comptabilisée » (#828), passe complète, trois lentilles B, E, A (Sonnet, lecture seule)

Tu es **une** des trois lentilles de la passe 2 de `bmad-code-review` de la story **34-4** de Kesh (issue #828 : la page de
réconciliation signale une transaction qui semble déjà comptabilisée, et demande confirmation avant *Accepter*, *Écriture
manuelle* et *Éclater*). Passe **complète** (périmètre entier de la story), modèle **Sonnet** (la P1 était Opus), contexte
frais. Travaille en français soigné. L'orchestrateur te dit laquelle tu es :

- **B — Blind Hunter** : le diff seul, sans la fiche ; défauts que le code porte en lui-même — nombre faux, requête SQL du
  second côté qui compte trop ou trop peu, ordre des paramètres liés, champ silencieusement à zéro, repli différent de son
  catalogue, réactivité Svelte, geste repris sur une sélection périmée, test qui passerait à vide.
- **E — Edge Case Hunter** : chaque branche et chaque frontière — `alreadyBooked` `null` / zéros / `k = 1` / `k > 1` / `m`
  seul / `k` et `m` ; hors CHF, montant nul, `-0.00` ; compte bancaire sans grand livre, archivé, **clos** ; comptes qui
  partagent un grand livre ; compte relié ailleurs ou délié **après** un rapprochement ; transaction rapprochée de même sens,
  de sens contraire, d'un montant voisin ; écriture contre-passée, contre-passation ; écriture rapprochée de **deux**
  transactions (invariant non défendu par une contrainte) ; règlement client à −5 et −6 centimes ; plafonds de 50 ;
  *Accepter* sur une sélection mixte, *Annuler*, Échap ; ligne `notice` signalée ; `limit=500` ; autre société ; locales.
- **A — Acceptance Auditor** : la fiche contre le code, AC par AC (AC-1 à AC-10), test par test, mutation par mutation
  (M1 à M9, journaux `mut-p1/`), banc AC-8, manuel `.tex`, `docs/api-external.md`, CHANGELOG, registre, captures ; et la
  remédiation de la P1, finding par finding (Change Log de la fiche, entrée du 2026-11-12).

## ⚠️ Regard appuyé : E1-1 et le choix C-e344-11

La P1 a relevé (E1-1, MEDIUM) que le second côté d'un virement dépendait du lien **actuel** d'un compte bancaire à son grand
livre. L'orchestrateur a arbitré : le prédicat ne doit plus en dépendre. Le développeur a constaté que la règle de
l'arbitrage, prise à la lettre (« une ligne sur le grand livre du compte courant »), ne corrigeait rien, et a retenu à la
place (C-e344-11, `e34-choix-autonomes-e344.md`) : **l'écriture est rapprochée d'une transaction d'un autre compte bancaire,
de sens contraire** (`bt.bank_account_id <> ? AND bt.amount = -m.movement`, constante `MATCHED_ELSEWHERE` de
`crates/kesh-db/src/repositories/reconciliation_link.rs`). Exerce ce choix sans complaisance :

1. Est-il juste pour chaque cas que la fiche couvre (AC-2 : virement de A vers B ; comptes partagés ; contre-passation) et
   pour le cas E1-1 (compte relié au 1022 puis délié) ?
2. Quel **faux positif** reste-t-il ? (Une écriture `D banque A / C …` rapprochée sur un autre compte d'une transaction de sens
   contraire sans être un virement ; une écriture à plus de deux lignes ; une écriture rapprochée de deux transactions ;
   montants opposés par coïncidence.) Lequel conseillerait « rejetez-la » à tort ?
3. Quel **faux négatif** ? (Virement dont les deux côtés diffèrent de quelques centimes, frais ; virement passé en deux
   écritures ; virement entre deux comptes qui partagent un grand livre.) Est-il dit au manuel ?
4. Le texte (b) et (c), le manuel (`sec:deja-comptabilisee`, l'encadré « Restent sans écriture… », § *Un virement*) et
   `api-external.md` disent-ils la règle neuve, sans reste de « relié à un autre compte du grand livre » qui serait faux ?
   (`/usr/bin/grep -nF "grand livre distinct" …`, `/usr/bin/grep -nF "journal_account_id" crates/kesh-db/src/repositories/reconciliation_link.rs`.)
5. La transmission à la 34-11b (fiche AC-5, F4-2, `kesh-gate-logs/e34-transmission-34-11b.md`) reste-t-elle vraie avec cette
   règle ?

## Le périmètre

```sh
git -C /home/gcorbaz/devel/kesh-e344 log --oneline f12ad73e..HEAD
git -C /home/gcorbaz/devel/kesh-e344 diff f12ad73e HEAD -- crates frontend docs CHANGELOG.md
git -C /home/gcorbaz/devel/kesh-e344 diff 46c9d755 HEAD -- crates frontend docs        # la remédiation P1 seule
```

Lis le diff **aplati**. Fichiers principaux : `crates/kesh-db/src/repositories/reconciliation_link.rs`,
`crates/kesh-api/src/routes/reconciliation.rs`, `crates/kesh-api/tests/reconciliation_link_entry_e2e.rs` (tests `s34_4_…`,
aides `forger_historique`, `rapprocher_ailleurs`, banc ignoré), `frontend/src/lib/features/reconciliation/already-booked.ts`,
`AlreadyBookedConfirmDialog.svelte`, `ReconciliationProposals.svelte`, `reconciliation.types.ts`, les tests
`ReconciliationProposals.test.ts`, `already-booked-i18n.test.ts`, les doublures `ModalSuccessStub*.test.svelte`,
`frontend/src/lib/shared/i18n-keys.test.ts`, l'E2E `frontend/tests/e2e/reconciliation-already-booked.spec.ts`, les quatre
`crates/kesh-i18n/locales/*/messages.ftl`, `docs/manual/fr/user-manual.tex`, `docs/api-external.md`, `CHANGELOG.md`.

Fiche : `/home/gcorbaz/devel/kesh-e344/_bmad-output/implementation-artifacts/34-4-transaction-deja-comptabilisee.md`
(lentille A ; B la lit **après** son examen à l'aveugle, pour classer). Registres : `e34-choix-autonomes-e342.md` et
`e34-choix-autonomes-e344.md` (C-e344-1 à 16). Rapports de la P1 : `/home/gcorbaz/devel/kesh-gate-logs/e344-review-p1-{B,E,A}.md`.
Journaux, sous `/home/gcorbaz/devel/kesh-gate-logs/e344/` : `rouges.log`, `p1/` (rouges, gates de la remédiation, chacun
préfixé du hash du commit joué), `mut-p1/M1.log` à `M9.log`, `banc-p1-apres.log` (et les bancs de la P0), `captures-p1/`.

## ⛔ Hors périmètre — ne pas le demander

#982 (fenêtre de dates) ; #827 (autre grand livre, ancien) ; #823 (rattachement des deux côtés) ; la réécriture des textes
(b) et (c) par la 34-11b ; #998, #999, #1000 ; le PDF du manuel (régénéré à l'intégration, C-e342-20) ; le format des
montants de la page (C-e344-15, issue demandée).

## Axes obligatoires

1. **Une seule règle** : les deux premiers nombres égalent-ils les listes du dialogue (mêmes constantes SQL, même tolérance) ?
   Le test d'égalité AC-8 couvre-t-il ce que la lecture par page réécrit (montant, tolérance, plafonds, société) ?
2. **Le second côté** : le regard appuyé ci-dessus.
3. **L'écran** : quatre cas, textes neutres en nombre, confirmation sur les trois gestes seulement, *Continuer* rouvre la
   bonne modale, Échap = *Annuler*.
4. **Le manuel et l'API** : contre le code (le PDF n'est pas régénéré dans la branche : dis-le en axe non exercé).
5. **Tests, mutations, banc, gates** : chaque mutation tuée pour la bonne raison ; le banc avec écritures rapprochées ;
   les gates de `p1/` au commit `e29a85ca`.
6. **Lecture seule** : `get_proposals` n'écrit rien ; aucune route d'écriture ne change.

## ⛔ Ground-truth obligatoire

Toute affirmation de présence ou d'absence d'un texte s'appuie sur une commande `/usr/bin/grep -nF "<chaîne exacte>"
<fichier>` (ou `git -C … show HEAD:<fichier> | /usr/bin/grep -nF …`), **dont tu copies la sortie**. Bloc multi-lignes :
`-A5`. (`grep` est une fonction vers `ugrep` : utilise **`/usr/bin/grep`**.)

## Sévérités

CRITICAL (écriture fausse possible, refus serveur introduit, perte de données), HIGH (AC faux, nombre qui diverge du dialogue,
conseil de rejet à tort sur un cas courant, test qui ne prouve rien), MEDIUM (incohérence code/fiche/manuel, repli divergent,
mutation tuée pour une mauvaise raison, manuel faux), LOW (rédaction, nit).

## Interdits — lecture seule stricte

Aucune commande qui écrit, compile ou exécute : pas de `cargo`, `npm`, `npx`, `make`, `latexmk`, `docker`, `curl`, `sqlx`,
`mariadb` ; **pas de `scripts/prepare-release.sh`** ni d'aucun script de `scripts/` ni de `kesh-gate-logs/` ; pas de
`git checkout`/`switch`/`restore`, `git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase` ; aucune
commande `gh` qui écrit ; pas de `claude`, `pkill`, `killall`, `rm`, `sed -i`, `perl -pi`, `tee`, **aucune redirection `>`
ni fichier temporaire**. **Tu n'écris qu'un seul fichier** : ton rapport.

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/e344-review-p2-<B|E|A>.md` (jamais sous `target/`) :

1. **Décompte** CRITICAL / HIGH / MEDIUM / LOW, recompté depuis ta liste.
2. **Findings** : identifiant (`B2-1`, `E2-1`, `A2-1`…), sévérité, fichier et ligne, sortie `grep -nF` copiée, correctif
   proposé.
3. **Axes exercés et non exercés** — liste explicite (le manuel, les captures, le PDF compris) ; un « 0 finding » sans
   cette liste ne compte pas.
