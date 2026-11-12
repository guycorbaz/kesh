# Revue de code P3 ciblée — 34-4 « transaction déjà comptabilisée » (#828) : la remédiation de la P2 (Haiku, une lentille, lecture seule)

Tu es la lentille **unique** de la passe 3 de `bmad-code-review` de la story **34-4** de Kesh. C'est une **passe ciblée**
(`CLAUDE.md`, § *La passe ciblée*) : une seule question — **la remédiation de la P2 a-t-elle introduit un défaut, ou laissé
un finding de la P2 sans effet ?** Contexte frais, français soigné.

## Le périmètre : le seul commit de code de la remédiation P2

```sh
git -C /home/gcorbaz/devel/kesh-e344 show --stat cf585bff
git -C /home/gcorbaz/devel/kesh-e344 show cf585bff -- crates frontend docs
```

Le diff est **un seul commit** : lis-le aplati, sans le confronter aux commits antérieurs ligne à ligne. Les findings de la P2
qu'il remédie sont dans `/home/gcorbaz/devel/kesh-gate-logs/e344-review-p2-{B,E,A}.md` ; le Change Log de la fiche
(`/home/gcorbaz/devel/kesh-e344/_bmad-output/implementation-artifacts/34-4-transaction-deja-comptabilisee.md`, entrée de la
remédiation P2) et le registre (`e34-choix-autonomes-e344.md`, C-e344-17 et C-e344-18) disent ce qui a été fait.

## Axes

1. **`confirmGesture`** (`frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte`) : la relecture de la ligne
   par identifiant — *Accepter* inchangé ? *Écriture manuelle* et *Éclater* ouvrent-ils la bonne modale avec la ligne relue ?
   Rien ne s'ouvre-t-il si la ligne a disparu ? `confirmTransactions` non vidé : la liste du dialogue suivant est-elle bien
   remplacée (`guard`) ? *Annuler* et Échap restent-ils sans effet ? Le test Vitest neuf prouve-t-il ce qu'il dit (un clic sur
   « Afficher aussi les plus anciennes » pendant la confirmation, puis *Continuer*) ?
2. **Les deux tests Rust neufs** (`crates/kesh-api/tests/reconciliation_link_entry_e2e.rs`,
   `s34_4_ac2_le_second_cote_d_un_debit`, `s34_4_ac2_rapprochee_sur_le_meme_compte_ne_compte_pas`) : assertent-ils ce que
   leur doc dit ? Le second passerait-il **à vide** (l'écriture est-elle vraiment rapprochée de la transaction du même compte,
   de montant opposé au mouvement sur le nouveau grand livre) ? La mutation **M10** (`kesh-gate-logs/e344/mut-p2/M10.log`)
   le fait-elle rougir pour la bonne raison ?
3. **Textes** : l'allemand de `crates/kesh-i18n/locales/de-CH/messages.ftl` (clés `…-already-booked-one`, `-other`,
   `-elsewhere`, `-confirm-intro`) — juste, sans ambiguïté, cohérent avec fr-CH ?
4. **Manuel** (`docs/manual/fr/user-manual.tex`) : les trois passages changés (« Le second côté d'un virement est signalé… »,
   § *Un virement entre deux de vos comptes*, dernier alinéa de `sec:deja-comptabilisee`) disent-ils la règle du code
   (`MATCHED_ELSEWHERE` : autre compte bancaire, montant opposé) sans reste de « grands livres distincts » ? Les faux négatifs
   (frais, arrondi, plusieurs écritures) et le doublon (deux transactions du même montant) sont-ils vrais contre le code ?
   Le PDF n'est pas régénéré dans la branche : ne le contrôle pas.
5. **Fiche** : les annotations datées des AC-1, AC-2, AC-5 et du tableau des mutations (M8, M10) disent-elles le code tel qu'il
   est ?

## ⛔ Ground-truth obligatoire

Toute affirmation de présence ou d'absence s'appuie sur `/usr/bin/grep -nF "<chaîne exacte>" <fichier>` (ou
`git -C /home/gcorbaz/devel/kesh-e344 show cf585bff:<fichier> | /usr/bin/grep -nF …`), **dont tu copies la sortie** ; bloc
multi-lignes : `-A5`. (`grep` est une fonction vers `ugrep` : utilise **`/usr/bin/grep`**.) Un finding CRITICAL ou HIGH qui
affirme l'absence d'un code attendu doit citer la sortie d'un `grep -nF` sur le fichier **final**, non sur le diff.

## Sévérités

CRITICAL (écriture fausse possible), HIGH (geste rouvert sur la mauvaise ligne, test qui ne prouve rien), MEDIUM (incohérence
code/manuel/fiche, mutation tuée pour une mauvaise raison), LOW (rédaction).

## Interdits — lecture seule stricte

Aucune commande qui écrit, compile ou exécute : pas de `cargo`, `npm`, `npx`, `make`, `docker`, `curl`, `sqlx`, `mariadb` ;
**pas de `scripts/prepare-release.sh`** ni d'aucun script de `scripts/` ni de `kesh-gate-logs/` ; pas de `git checkout`/
`switch`/`restore`, `git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase` ; aucune commande `gh` qui
écrit ; pas de `claude`, `pkill`, `killall`, `rm`, `sed -i`, `perl -pi`, `tee`, **aucune redirection `>` ni fichier
temporaire**. **Tu n'écris qu'un seul fichier** : ton rapport.

## Rapport

Écris-le dans `/home/gcorbaz/devel/kesh-gate-logs/e344-review-p3-ciblee.md` :

1. **Décompte** CRITICAL / HIGH / MEDIUM / LOW, recompté depuis ta liste.
2. **Findings** : identifiant (`C3-1`…), sévérité, fichier et ligne, sortie `grep -nF` copiée, correctif proposé.
3. **Axes exercés et non exercés** — liste explicite ; un « 0 finding » sans cette liste ne compte pas.
