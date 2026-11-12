# Revue de code P2 ciblée — 34-3 « Écriture manuelle » (#967), une lentille (Haiku, lecture seule)

Tu es la **passe ciblée** de la boucle `bmad-code-review` de la story **34-3** de Kesh. La passe 1 (Opus, trois lentilles)
a rendu 0 CRITICAL, 0 HIGH, 0 MEDIUM et 15 LOW. Une partie de ces LOW a été remédiée par **un seul commit de code**. C'est
ce commit, et lui seul, que tu relis. Tu cherches les défauts qu'il **introduit**, et tu vérifies qu'il fait ce qu'il
annonce. Contexte frais. Travaille en français soigné.

## Le périmètre — le seul commit de remédiation

```sh
git -C /home/gcorbaz/devel/kesh-e3403 show --stat 584a1183
git -C /home/gcorbaz/devel/kesh-e3403 show 584a1183
```

Lis ce diff, d'un seul commit. Pour comprendre le contexte d'une ligne, tu peux lire le fichier entier à `HEAD`
(`git -C … show HEAD:<fichier>`).

Références, à lire seulement pour classer :
- les rapports de la passe 1 : `/home/gcorbaz/devel/kesh-gate-logs/e3403-review-p1-{B,E,A}.md` ;
- la fiche : `_bmad-output/implementation-artifacts/34-3-ecriture-manuelle.md`, AC-4 et AC-7, et l'entrée « revue de code
  P1 » de son Change Log ;
- le registre : `e34-choix-autonomes-e3403.md`, C-e3403-8 (texte de l'avis créanciers, décidé par l'orchestrateur) et
  C-e3403-9.

## Ce que le commit annonce, à vérifier point par point

1. **B1-1** : `claimRoleOfId(accounts, accountId)` (`reconciliation-counterparties.ts`) est la **seule** règle d'affichage
   de l'avis. `CounterpartyClaimNotice.svelte` et la rangée d'*Éclater* (`TransactionSplitModal.svelte`) l'emploient, et
   plus aucun site ne recalcule `claimRoleOf(... .find(...))`. Le comportement doit rester inchangé : même liste, même
   rôle. Il a son test (`reconciliation-counterparties.test.ts`).
2. **B1-2, E1-3, A1-3** : trois Vitest neufs dans `TransactionSplitModal.test.ts` :
   - l'avis suit sa ligne après une suppression (`data-line` du nouvel indice) ;
   - deux lignes sur le compte débiteurs portent deux avis ;
   - le `colspan` vaut 4 sans projets et 5 avec, égal au nombre de colonnes de l'en-tête.

   Ces tests prouvent-ils vraiment ce qu'ils disent, ou passeraient-ils à vide ? Une assertion qui précède l'état
   qu'elle croit observer passe à vide, de même qu'un `waitFor` qui réussit d'emblée.
3. **B1-6** : le test « un compte de TVA n'avertit pas » prouve d'abord que le choix a pris (le champ montre `1170`, la
   validation est active), puis vérifie l'absence de l'avis.
4. **B1-3, E1-4, A1-1** : l'E2E `reconciliation-manual-entry.spec.ts` pose une **ancre positive** avant d'asserter
   l'absence du compte de la banque. Avec la saisie de `1`, il vérifie `1100` présent et le compte de la banque absent.
   L'ancre dépend-elle d'un compte que le préréglage `with-company` sème vraiment (`crates/kesh-db/src/test_fixtures.rs`) ?
   Le filtre `hasText` peut-il confondre deux numéros ? La mutation M2 a été rejouée sur l'E2E :
   `/home/gcorbaz/devel/kesh-gate-logs/e3403/mut/M2-e2e.log` et `M2-e2e-run.log`. Le rouge vient-il de la bonne
   assertion ?
5. **Textes** :
   - **B1-4** : de « Die Rechnung bliebe … » ;
   - **A1-4** : en « This is the accounts receivable/payable account. » ;
   - **B1-5 / C-e3403-8** : l'avis créanciers reformulé, dont le sens est « saisissez la facture et validez-la depuis sa
     fiche, sans y enregistrer de paiement : le paiement est cette transaction, et la facture sera proposée ici ».

   Vérifie les quatre catalogues (`crates/kesh-i18n/locales/*/messages.ftl`), le repli de
   `CounterpartyClaimNotice.svelte`, la constante `AVIS_CREANCIERS` de `ManualMatchModal.test.ts` et le manuel
   `docs/manual/fr/user-manual.tex`. Le repli est-il **identique** au catalogue fr, octet pour octet (apostrophes
   comprises) ? Les traductions disent-elles le même sens, sans contresens ? Un ancien texte reste-t-il quelque part ?
6. **E1-1** : le manuel dit qu'ouverte sans rien taper, la liste ne montre que les premiers comptes (vingt au plus, dans
   l'ordre des numéros) et qu'il faut taper le numéro ou une partie du nom. C'est vrai si
   `frontend/src/lib/features/journal-entries/AccountAutocomplete.svelte` applique `slice(0, 20)` et si la liste du
   serveur est triée par numéro.

## Garde de l'AC-6

Rejoue la garde de la fiche (AC-6, motif copié de la fiche) en lecture :
`git -C /home/gcorbaz/devel/kesh-e3403 grep -niE '<motif>' HEAD -- crates/kesh-i18n frontend/src frontend/tests 'docs/manual/fr/*.tex'`.
Elle doit rendre exactement les cinq survivants annoncés.

## ⛔ Ground-truth obligatoire

Toute affirmation de présence ou d'absence s'appuie sur une commande `/usr/bin/grep -nF "<chaîne exacte>" <fichier>`
(ou `git -C … show HEAD:<fichier> | /usr/bin/grep -nF …`), **dont tu copies la sortie**. Pour un bloc sur plusieurs
lignes, utilise `-A5`. Dans ce shell, `grep` est une fonction vers `ugrep` : emploie **`/usr/bin/grep`**.

## Sévérités

- **CRITICAL** : écriture fausse possible, compte de la banque accepté.
- **HIGH** : AC faux, test qui ne prouve rien sur un point annoncé.
- **MEDIUM** : régression introduite par le commit, repli divergent du catalogue, texte faux, manuel faux.
- **LOW** : rédaction.

## Interdits — lecture seule stricte

Tu ne lances aucune commande qui écrit, compile ou exécute :
- pas de `cargo`, `npm`, `npx`, `make`, `latexmk`, `docker`, `curl`, `sqlx` ni `mariadb` ;
- **pas de `scripts/prepare-release.sh`**, ni aucun script de `scripts/` ou de `kesh-gate-logs/` ;
- pas de `git checkout`/`switch`, `git reset`, `git stash`, `git commit`, `git push`, `git tag` ni `git rebase` ;
- aucune commande `gh` qui écrit ;
- pas de `claude`, `pkill`, `killall`, `rm`, `sed -i`, `perl -pi` ni `tee` ;
- **aucune redirection `>` ni fichier temporaire**.

**Tu n'écris qu'un seul fichier** : ton rapport.

## Rapport

Écris-le dans `/home/gcorbaz/devel/kesh-gate-logs/e3403-review-p2-ciblee.md` (jamais sous `target/`). Il contient :

1. **Décompte** CRITICAL / HIGH / MEDIUM / LOW, recompté depuis ta liste.
2. **Findings** : pour chacun, un identifiant (`C2-1`…), la sévérité, le fichier et la ligne, la sortie `grep -nF` copiée
   et le correctif proposé.
3. **Axes exercés et non exercés** : liste explicite des points 1 à 6 et de la garde. Un « 0 finding » sans cette liste
   ne compte pas.
