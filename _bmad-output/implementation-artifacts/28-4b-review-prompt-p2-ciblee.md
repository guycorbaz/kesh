# Prompt — revue de code P2 **ciblée**, Story 28-4b (remédiation de la revue P1)

*Versionné le 2026-10-14. **Une seule lentille** (Sonnet), contexte frais, lecture seule, braquée sur **la seule
remédiation** de la revue P1 (§ *La passe ciblée* du `CLAUDE.md`) : la sévérité ne stagne pas, elle se déplace vers
ce qu'on vient d'écrire.*

Dépôt (worktree) `/home/gcorbaz/devel/kesh-28-4b`, branche `story/28-4b`.
**Le diff à revoir** : `git diff da9fde6a 25fbeede -- . ':(exclude)_bmad-output'` — `da9fde6a` est le dernier commit
d'avant la remédiation ; `929bd0ef` porte **tout le code** de la remédiation, `8a6a96ad` le manuel, `25fbeede` les PDF.
⚠️ Pathspec `':(exclude)…'` et **non** `':!…'`. Contexte : les trois rapports P1
(`/home/gcorbaz/devel/kesh-gate-logs/28-4b-review-p1-{B,E,A}.md`), la fiche
`_bmad-output/implementation-artifacts/28-4-cloture-compte-bancaire.md` (partie b, Dev Agent Record § *28-4b* et son
annotation de la revue P1), les choix **C-28-4-47 à C-28-4-50** au registre `epic-28-choix-autonomes.md`, la campagne
`kesh-gate-logs/28-4b-mutations-p1.log` (7 / 7 tuées). Issues : #642 (fermée par la PR), #651 (`refs`).

⛔ **DONNÉES PERSONNELLES — DÉPÔT PUBLIC.** N'ouvre aucun fichier hors du dépôt et de
`/home/gcorbaz/devel/kesh-gate-logs/`.

## La lentille : chasse aux régressions de la remédiation

Pour chaque correctif, deux questions : **a-t-il corrigé ce que le finding disait**, et **qu'a-t-il cassé à côté** ?

1. **A-M1 — libellés it/en et garde G11-bis** (`kesh-i18n/locales/{it-CH,en-CH}/messages.ftl`,
   `kesh-i18n/src/loader.rs`, `docs/i18n-glossaire.md`). « Chiudi il conto » / « Close account » : un autre libellé
   neuf de la 28-4b reste-t-il un verbe nu de panneau (`grep -nE "^bank-accounts-.* = (Close|Chiudi|Schliessen|Fermer)$"`) ?
   Le bouton de ligne et la confirmation disent-ils désormais **la même chose** en italien, et est-ce un défaut ? La
   garde G11-bis : rougit-elle vraiment sur le verbe nu de **chaque** locale, ou seulement sur la première rencontrée ?
   Son ensemble « panneaux » est-il non vide dans les quatre locales ? Le glossaire dit-il vrai au catalogue (clés
   citées comme précédents, présentes ? `grep -nF`) ?
2. **B-1 = E-4 — `toggleAndReload`** (`frontend/src/routes/(app)/bank-accounts/+page.svelte`). `closeForm()` en tête
   des deux bascules : que perd l'utilisateur qui avait un **formulaire de création ou de modification** ouvert et
   clique une bascule (saisie effacée sans avertissement) ? Est-ce acceptable, et le dit-on ? L'ordre `closeForm()` →
   `flip()` → `loading` → `reload()` laisse-t-il un état incohérent si `reload()` échoue ? La refactorisation de
   `handleToggleArchived` garde-t-elle son comportement d'avant (test existant ?) ?
3. **B-2 — `refreshAfterVersionConflict`**. Le `reload()` dans le `catch` s'exécute **avant** `finally { submitting =
   false }` : le bouton reste-t-il désactivé pendant le rechargement, et est-ce voulu ? `reload()` ne lève jamais
   (`allSettled`) — vrai ? Le code compare `err.code` : un refus `409` d'un autre code (déjà clos, pas clos) est-il bien
   exclu ? Après le rechargement, si le compte a disparu de la liste (clos par un autre onglet **et** clos masqués), que
   devient le dialogue ?
4. **A-L6 — liste vide** : la condition `includeClosed && includeArchived` choisit-elle le bon message dans les
   **quatre** combinaisons ? Les quatre traductions de `bank-accounts-labels-empty-hidden` disent-elles la même chose,
   et l'allemand évite-t-il « geschlossen » (G11) ? `sitesTotal` 2104 recompté (`grep -o "i18nMsg("` sur la page,
   69 → 70) ?
5. **E-1 — le test de la date locale** : `process.env.TZ` modifié en cours de processus est-il réellement relu par
   `Date` sous Vitest/jsdom (le test vérifie son montage : jour local 14, jour UTC 13 — ces deux assertions
   suffisent-elles à prouver que le fuseau a pris) ? La restauration (`finally`) est-elle complète, et un test suivant
   peut-il hériter d'une horloge figée ou du fuseau ? `toFake: ['Date']` laisse-t-il bien les minuteries réelles aux
   attentes de testing-library ?
6. **Les tests neufs prouvent-ils ce qu'ils nomment** (E-2, E-3, bascules, liste vide) : chacun rougit-il sous la
   mutation qui le vise (journal `28-4b-mutations-p1.log`) ? Cherche **une mutation plausible de la remédiation qui
   resterait verte**.
7. ⛔ **Manuel, PDF compris** (`docs/manual/fr/user-manual.tex`, diff `8a6a96ad`) : les phrases corrigées (A-L3,
   A-L4 : archivage, limites — *Annuler le rapprochement*, *Annuler le rejet*, *Annuler la facture* —, renvoi du solde,
   lot confirmé sur un compte clos ; A-L6 : liste vide) disent-elles vrai au code ? Une phrase neuve est-elle
   **fausse** (ex. : *Annuler la facture* d'une facture fournisseur payée par ce compte est-il bien un bouton affiché
   puis refusé ?) ? Contrôle le PDF **aplati** :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` vers
   `/home/gcorbaz/devel/kesh-gate-logs/28-4b-review-p2-ciblee-user-manual-flat.txt` (⚠️ ligature « ﬀ », apostrophe
   « ’ »).
8. **Commentaires** (B-3 angle mort, B-4, E-5) : disent-ils vrai ?

## Ce que tu rends

Ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/28-4b-review-p2-ciblee.md`. Findings avec sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie, ou code relu), correction proposée. Pour
tout finding affirmant qu'un code ou un texte est absent ou présent : la sortie d'un `grep -nF` copiée. Dis, pour
chaque finding, s'il vient **de la remédiation** ou d'avant.
⛔ **La liste des axes exercés ET non exercés** (les huit ci-dessus) — un « 0 finding » sans elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport et du texte aplati du PDF, dans `/home/gcorbaz/devel/kesh-gate-logs/` ;
aucune commande qui écrit dans le dépôt ou dans une base, ni qui compile ou exécute : `scripts/*` (dont
`scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`), `make`, `latexmk`, `xelatex`,
⛔ **`git checkout`** (sous aucune forme, pas même `git checkout -- <fichier>` ni `git checkout <sha>`), `git
commit`/`add`/`switch`/`stash`/`reset`/`rebase`/`restore`/`push`/`tag`, `sqlx`, `cargo` (aucune sous-commande), `npm`,
`npx`, `node`, `deno`, `bun`, `python`/`python3` (pas même les scripts de mutations), `perl`, `ruby`, `php`, `sed -i`,
`awk` en écriture, tout autre interpréteur, `mariadb`, `mysql`, `docker`, `curl`, `wget`, `gh` en écriture,
`pkill`/`kill`, aucune requête SQL. Pour lire une version passée d'un fichier : `git show <sha>:<chemin>`. Autorisés :
lecture, `grep`, `sed -n`, `wc`, `sort`, `git log`/`show`/`diff`, `gh issue view`, `pdftotext` vers
`/home/gcorbaz/devel/kesh-gate-logs/`, l'ouverture des PNG de `kesh-gate-logs/28-4b-captures/`.
