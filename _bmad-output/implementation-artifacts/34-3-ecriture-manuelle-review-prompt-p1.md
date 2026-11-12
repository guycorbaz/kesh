# Revue de code P1 — 34-3 « Écriture manuelle » (#967), trois lentilles B, E, A (Sonnet ou Opus, lecture seule)

Tu es **une** des trois lentilles de la passe 1 de `bmad-code-review` de la story **34-3** de Kesh (issue #967 :
« Écriture manuelle » sur tout compte imputable, sauf celui de la banque). Contexte frais. Travaille en français soigné.
Une première passe n'est jamais confiée à Haiku. L'orchestrateur te dit laquelle tu es :

- **B — Blind Hunter** : le diff seul, sans la fiche ; cherche les défauts que le code porte en lui-même (liste proposée
  fausse, compte de la banque proposé, avis affiché à tort ou absent, réactivité Svelte — `$derived`, ligne d'éclatement
  réordonnée ou retirée —, texte d'un repli différent de son catalogue, test qui passerait à vide).
- **E — Edge Case Hunter** : chaque branche et chaque frontière — `bankLedgerAccountId` `null` / présent / absent de la
  liste ; compte archivé, non imputable, de chaque classe 1 à 9 ; rôle `Receivable`, `Payable`, de TVA, `null` ; compte
  choisi puis changé, puis effacé ; *Éclater* : deux lignes sur le même compte débiteurs, ligne retirée (les clés `(i)` du
  `{#each}`), ajout de ligne, colonne des projets présente ou non (`colspan`) ; plus de 20 comptes (l'autocomplétion n'en
  montre que 20 sans saisie) ; locale de/en/it.
- **A — Acceptance Auditor** : la fiche contre le code, AC par AC (AC-1 à AC-8), test par test, mutation par mutation
  (M1 à M5 et M4-bis, journaux cités ci-dessous), manuel `.tex`, CHANGELOG, registre, captures.

## Le périmètre

```sh
git -C /home/gcorbaz/devel/kesh-e3403 log --oneline 7f817792..HEAD
git -C /home/gcorbaz/devel/kesh-e3403 diff 7f817792 HEAD -- crates frontend docs CHANGELOG.md
```

Lis le diff **aplati** (un seul diff de `7f817792` à `HEAD`). Fichiers principaux :
`frontend/src/lib/features/reconciliation/reconciliation-counterparties.ts` et son test (neufs),
`CounterpartyClaimNotice.svelte` (neuf), `ManualMatchModal.svelte`, `TransactionSplitModal.svelte`,
`ReconciliationProposals.svelte`, `failed-proposal-label.ts`, `ecriture-manuelle-i18n.test.ts` (neuf), les tests
`ManualMatchModal.test.ts`, `TransactionSplitModal.test.ts`, `ReconciliationProposals.test.ts`,
`failed-proposal-label.test.ts`, `ModalSuccessStub.test.svelte`, l'E2E neuf
`frontend/tests/e2e/reconciliation-manual-entry.spec.ts`, les quatre `crates/kesh-i18n/locales/*/messages.ftl`, deux
commentaires Rust (`crates/kesh-api/src/routes/reconciliation.rs`, `crates/kesh-reconciliation/src/manual.rs`),
`docs/manual/fr/user-manual.tex`, `CHANGELOG.md`.

Fiche : `/home/gcorbaz/devel/kesh-e3403/_bmad-output/implementation-artifacts/34-3-ecriture-manuelle.md` (lentille A ;
B la lit **après** son examen à l'aveugle, pour classer). Registres : `e34-choix-autonomes-e342.md` (choix de la fiche,
C-e342-1 à C-e342-5) et `e34-choix-autonomes-e3403.md` (choix du développement, C-e3403-1 à C-e3403-7). Journaux :
rouges `/home/gcorbaz/devel/kesh-gate-logs/e3403/rouges.log`, mutations
`/home/gcorbaz/devel/kesh-gate-logs/e3403/mut/*.log`, garde de l'AC-6 avant et après
(`/home/gcorbaz/devel/kesh-gate-logs/e3403/garde-ac6-{avant,apres}.log`), gates `/home/gcorbaz/devel/kesh-gate-logs/e3403/gate-*.log`,
E2E `/home/gcorbaz/devel/kesh-gate-logs/e3403/e2e-*.log`, captures `/home/gcorbaz/devel/kesh-gate-logs/e3403/captures/*.png`
(le Dev Agent Record de la fiche les cite et dit ce qui a tourné).

## ⛔ Hors périmètre — ne pas le demander

Les règles d'affectation restent aux comptes de charges et de produits (#973) ; l'avis ne couvre ni les comptes de TVA
(#981) ni un compte débiteurs ou créanciers sans rôle posé (choix « rôle explicite », Story 14-3a) ; aucun code serveur
exécutable ne change (AC-5) ; le PDF du manuel utilisateur se régénère **à l'intégration** (AC-8 : il est touché par
34-3, 34-4, 34-9 et 34-10) — son absence de la branche n'est pas un finding ; le libellé d'audit
`audit-log-action-reconciliation-manual-matched` ne change pas (C-e342-3).

## Axes obligatoires

1. **La liste proposée** : `counterpartyAccounts` est-elle la seule règle (DRY), employée par les deux dialogues ? Le
   compte de la banque est-il écarté partout, par identifiant (jamais par numéro) ? `TransactionSplitModal` reçoit-il
   bien `bankLedgerAccountId` de `ReconciliationProposals` ?
2. **L'avis** (AC-4) : deux textes exacts (fiche, AC-4), quatre locales, `data-testid`, `data-role` ; absent pour un
   autre rôle et sans rôle ; disparaît au changement de compte ; ne bloque pas la validation. Dans *Éclater* : sous la
   **seule** ligne concernée (C-e3403-1), juste après suppression d'une ligne.
3. **Les textes** (AC-1, AC-6) : la garde de l'AC-6, copiée de la fiche, rejouée par toi en lecture
   (`git -C … grep -niE '<motif>' -- crates/kesh-i18n frontend/src frontend/tests 'docs/manual/fr/*.tex'`) rend-elle
   exactement les cinq survivants ? Les replis Svelte/TS égalent-ils le catalogue fr ? Les traductions de/en/it disent-
   elles le geste neuf, sans contresens ?
4. **Le manuel** — `docs/manual/fr/user-manual.tex` : § *Écriture manuelle* (`\label{sec:ecriture-manuelle}`), l'encadré
   « Dépense déjà réglée », le § *Éclatement*, « La contrepartie n'est jamais le compte de la banque », § *Application
   des règles*, la FAQ « aucune correspondance » (deux conduites client / fournisseur), § *Affecter un projet*. Dit-il vrai
   contre le code (liste proposée, avis, rôles, second côté d'un virement, C-e3403-5) ? Plus aucune mention de « classes
   5, 6 et 7 » pour ces dialogues ? (`/usr/bin/grep -nF "classes 5, 6 et 7" docs/manual/fr/user-manual.tex`.) Le PDF
   n'est pas régénéré dans la branche : ne le contrôle pas, dis-le dans tes axes non exercés.
5. **Tests et mutations** : chaque mutation est-elle tuée par le test que la fiche nomme, pour la bonne raison (lis les
   `mut/*.log`) ? Le test d'ensemble compare-t-il un **ensemble** (non un cardinal) ? L'E2E vérifie-t-il l'écriture par
   l'API (compte, sens, montant) et le refus d'offrir le compte de la banque ? Un test passe-t-il à vide ?
6. **Serveur** : seuls deux commentaires changent dans `crates/` (hors catalogues) — vérifie qu'aucune ligne exécutable
   ne bouge (`git -C … diff 7f817792 HEAD -- crates/kesh-api/src crates/kesh-reconciliation/src`).

## ⛔ Ground-truth obligatoire

Toute affirmation de présence ou d'absence d'un texte s'appuie sur une commande
`/usr/bin/grep -nF "<chaîne exacte>" <fichier>` (ou `git -C … show HEAD:<fichier> | /usr/bin/grep -nF …`), **dont tu
copies la sortie**. Bloc multi-lignes : `-A5`. (`grep` est une fonction vers `ugrep` : utilise **`/usr/bin/grep`**.)

## Sévérités

CRITICAL (écriture fausse possible, compte de la banque accepté en contrepartie, perte de données), HIGH (AC faux, avis
absent sur un compte débiteurs ou créanciers au rôle posé, test qui ne prouve rien), MEDIUM (incohérence
code/fiche/manuel, repli divergent, mutation tuée pour une mauvaise raison, manuel faux), LOW (rédaction, nit).

## Interdits — lecture seule stricte

Aucune commande qui écrit, compile ou exécute : pas de `cargo`, `npm`, `npx`, `make`, `latexmk`, `docker`, `curl`,
`sqlx`, `mariadb` ; **pas de `scripts/prepare-release.sh`** ni d'aucun script de `scripts/` ni de `kesh-gate-logs/` ;
pas de `git checkout`/`switch`, `git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase` ; aucune
commande `gh` qui écrit ; pas de `claude`, `pkill`, `killall`, `rm`, `sed -i`, `perl -pi`, `tee`, **aucune redirection
`>` ni fichier temporaire**. **Tu n'écris qu'un seul fichier** : ton rapport.

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/e3403-review-p1-<B|E|A>.md` (jamais sous `target/`) :

1. **Décompte** CRITICAL / HIGH / MEDIUM / LOW, recompté depuis ta liste.
2. **Findings** : identifiant (`B1-1`, `E1-1`, `A1-1`…), sévérité, fichier et ligne, sortie `grep -nF` copiée, correctif
   proposé.
3. **Axes exercés et non exercés** — liste explicite (le manuel, les captures, le PDF compris) ; un « 0 finding » sans
   cette liste ne compte pas.
