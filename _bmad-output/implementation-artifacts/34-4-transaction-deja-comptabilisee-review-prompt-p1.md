# Revue de code P1 — 34-4 « transaction déjà comptabilisée » (#828), trois lentilles B, E, A (Sonnet ou Opus, lecture seule)

Tu es **une** des trois lentilles de la passe 1 de `bmad-code-review` de la story **34-4** de Kesh (issue #828 : la page de
réconciliation signale une transaction qui semble déjà comptabilisée, et demande confirmation avant *Accepter*, *Écriture
manuelle* et *Éclater*). Contexte frais. Travaille en français soigné. Une première passe n'est jamais confiée à Haiku.
L'orchestrateur te dit laquelle tu es :

- **B — Blind Hunter** : le diff seul, sans la fiche ; cherche les défauts que le code porte en lui-même — nombre faux (une
  seconde règle au lieu des fonctions du dialogue, paramètres différents), requête SQL du second côté qui compte trop ou trop
  peu (comptes partagés, contre-passation, autre société), champ silencieusement à zéro sur une erreur, texte d'un repli
  différent de son catalogue, réactivité Svelte (`pendingGesture`, `confirmTransactions`, `{@const}` dans le `{#each}`),
  geste repris sur une sélection périmée, test qui passerait à vide.
- **E — Edge Case Hunter** : chaque branche et chaque frontière — `alreadyBooked` `null` / trois zéros / `k = 1` / `k > 1` /
  `m` seul / `k` et `m` ; transaction hors CHF, de montant nul, `-0.00` ; compte bancaire sans compte du grand livre, archivé,
  clos ; deux comptes bancaires sur le même grand livre ; compte bancaire délié après un rapprochement (`<=>`, C-e344-2) ;
  écriture contre-passée, contre-passation ; règlement client à −5 et −6 centimes ; plafond de 50 ; *Accepter* sur une
  sélection mixte (signalée et non signalée), *Annuler* puis *Accepter* de nouveau, fermeture du dialogue par Échap ; une
  ligne `notice` (lot en cours) signalée ; `limit=500` ; locale de/en/it.
- **A — Acceptance Auditor** : la fiche contre le code, AC par AC (AC-1 à AC-10), test par test, mutation par mutation
  (M1 à M8, journaux cités ci-dessous), banc AC-8 (chiffres et conclusion), manuel `.tex`, `docs/api-external.md`,
  CHANGELOG, registre, captures.

## Le périmètre

```sh
git -C /home/gcorbaz/devel/kesh-e344 log --oneline f12ad73e..HEAD
git -C /home/gcorbaz/devel/kesh-e344 diff f12ad73e HEAD -- crates frontend docs CHANGELOG.md
```

Lis le diff **aplati** (un seul diff de `f12ad73e` à `HEAD`). Fichiers principaux :
`crates/kesh-db/src/repositories/reconciliation_link.rs` (mode `MatchedElsewhere` du constructeur `entry_sql`,
`find_matched_elsewhere`, `count_already_booked`, test unitaire d'égalité des textes),
`crates/kesh-api/src/routes/reconciliation.rs` (`get_proposals`, `AlreadyBooked`),
`crates/kesh-api/tests/reconciliation_link_entry_e2e.rs` (tests `s34_4_…`, banc ignoré
`banc_cout_propositions_deja_comptabilisees`), `frontend/src/lib/features/reconciliation/already-booked.ts` (neuf),
`AlreadyBookedConfirmDialog.svelte` (neuf), `ReconciliationProposals.svelte`, `reconciliation.types.ts`, les tests
`ReconciliationProposals.test.ts`, `already-booked-i18n.test.ts` (neuf) et les fixtures des trois modales, l'E2E neuf
`frontend/tests/e2e/reconciliation-already-booked.spec.ts`, les quatre `crates/kesh-i18n/locales/*/messages.ftl`,
`docs/manual/fr/user-manual.tex`, `docs/api-external.md`, `CHANGELOG.md`.

Fiche : `/home/gcorbaz/devel/kesh-e344/_bmad-output/implementation-artifacts/34-4-transaction-deja-comptabilisee.md`
(lentille A ; B la lit **après** son examen à l'aveugle, pour classer). Registres : `e34-choix-autonomes-e342.md` (choix
de la fiche, C-e342-5 à C-e342-9, C-e342-20 à C-e342-23) et `e34-choix-autonomes-e344.md` (choix du développement).
Journaux, sous `/home/gcorbaz/devel/kesh-gate-logs/e344/` : rouges `rouges.log` (et `rouges-backend.log`,
`rouges-vitest.log`), bancs `banc-avant.log` et `banc-apres.log`, mutations `mut/*.log`, gates `gate-*.log`, E2E
`e2e-*.log`, captures `captures/*.png` (le Dev Agent Record de la fiche les cite et dit ce qui a tourné).

## ⛔ Hors périmètre — ne pas le demander

Une fenêtre de dates pour le signal (#982, ouverte) ; un paiement passé sur un autre compte du grand livre, ou un ancien
(#827) ; le rattachement des deux côtés d'un virement à une même écriture (#823) ; la réécriture des textes (b) et (c) et de
leur condition par la 34-11b (fiche, AC-5, F4-2) ; le sélecteur de comptes à 20 entrées (#998), l'avis débiteurs/créanciers
qui ignore le sens (#999), le compte sorti de la liste resté envoyé (#1000) ; le PDF du manuel utilisateur se régénère **à
l'intégration** (AC-9, C-e342-20) — son absence de la branche n'est pas un finding.

## Axes obligatoires

1. **Une seule règle** (AC-1, C-v017-3-7) : `settlementCount` et `entryCount` sont-ils les longueurs de
   `find_link_candidates` et `find_entry_candidates`, avec les **mêmes** paramètres que les routes du dialogue (compte du
   grand livre, `link_amount_tolerance()`) ? Le `null` tombe-t-il exactement hors CHF, au montant nul, sans compte du grand
   livre ? Une erreur de lecture fait-elle échouer la route (AC-3) ?
2. **Le second côté** (AC-2) : le texte SQL du mode `MatchedElsewhere` ne diffère-t-il de la liste que par le prédicat de
   la condition 4 (le test unitaire le prouve-t-il, ou passerait-il à vide) ? L'ordre des paramètres liés correspond-il aux
   `?` du texte, dans les trois modes ? Le prédicat « autre compte bancaire, autre grand livre » est-il juste (`<=>`,
   C-e344-2) ?
3. **L'écran** (AC-5, AC-6) : quatre cas de texte, clés `-one`/`-other` choisies par le nombre, aucun sélecteur Fluent,
   aucun numéro d'issue, rien pour un compte nul ; confirmation sur les trois gestes et seulement sur eux ; *Annuler* n'appelle
   rien ; *Continuer* reprend le geste (lot entier pour *Accepter*). Les replis égalent-ils le catalogue fr, et les
   traductions de/en/it disent-elles la même chose ?
4. **Le manuel** — `docs/manual/fr/user-manual.tex` : le paragraphe neuf `\label{sec:deja-comptabilisee}`, l'encadré
   « Dépense déjà réglée » (qui disait « l'écran ne le signale pas »), la phrase ajoutée à « Restent sans écriture… », le
   § *Un virement entre deux de vos comptes*. Disent-ils vrai contre le code (conditions, plafond de 50, bruit d'un montant
   récurrent et de l'écriture d'ouverture, cas non signalés) ? Le PDF n'est pas régénéré dans la branche : ne le contrôle
   pas, dis-le dans tes axes non exercés. `docs/api-external.md` : le paragraphe « Déjà comptabilisée ».
5. **Tests, mutations, banc** : chaque mutation est-elle tuée par le test que la fiche nomme, pour la bonne raison (lis les
   `mut/*.log`) ? Les tests AC-1 comparent-ils le champ aux **routes** du dialogue, et non à des constantes seules ? L'E2E
   lit-il le texte de l'avertissement servi par le vrai serveur ? Le banc mesure-t-il ce que l'AC-8 demande, et la
   conclusion « bonne pente / mauvaise pente » se lit-elle dans les chiffres ?
6. **Lecture seule** (AC-3, AC-4) : `get_proposals` n'écrit rien ; aucune route d'écriture ne change
   (`git -C … diff f12ad73e HEAD -- crates/kesh-api/src/routes/reconciliation.rs`).

## ⛔ Ground-truth obligatoire

Toute affirmation de présence ou d'absence d'un texte s'appuie sur une commande
`/usr/bin/grep -nF "<chaîne exacte>" <fichier>` (ou `git -C … show HEAD:<fichier> | /usr/bin/grep -nF …`), **dont tu
copies la sortie**. Bloc multi-lignes : `-A5`. (`grep` est une fonction vers `ugrep` : utilise **`/usr/bin/grep`**.)

## Sévérités

CRITICAL (écriture fausse possible, refus serveur introduit, perte de données), HIGH (AC faux, nombre qui diverge du dialogue,
confirmation absente sur un des trois gestes, test qui ne prouve rien), MEDIUM (incohérence code/fiche/manuel, repli
divergent, mutation tuée pour une mauvaise raison, manuel faux), LOW (rédaction, nit).

## Interdits — lecture seule stricte

Aucune commande qui écrit, compile ou exécute : pas de `cargo`, `npm`, `npx`, `make`, `latexmk`, `docker`, `curl`,
`sqlx`, `mariadb` ; **pas de `scripts/prepare-release.sh`** ni d'aucun script de `scripts/` ni de `kesh-gate-logs/` ;
pas de `git checkout`/`switch`/`restore`, `git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase` ;
aucune commande `gh` qui écrit ; pas de `claude`, `pkill`, `killall`, `rm`, `sed -i`, `perl -pi`, `tee`, **aucune
redirection `>` ni fichier temporaire**. **Tu n'écris qu'un seul fichier** : ton rapport.

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/e344-review-p1-<B|E|A>.md` (jamais sous `target/`) :

1. **Décompte** CRITICAL / HIGH / MEDIUM / LOW, recompté depuis ta liste.
2. **Findings** : identifiant (`B1-1`, `E1-1`, `A1-1`…), sévérité, fichier et ligne, sortie `grep -nF` copiée, correctif
   proposé.
3. **Axes exercés et non exercés** — liste explicite (le manuel, les captures, le PDF compris) ; un « 0 finding » sans
   cette liste ne compte pas.
