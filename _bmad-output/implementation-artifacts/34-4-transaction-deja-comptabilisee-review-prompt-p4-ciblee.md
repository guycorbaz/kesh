# Revue P4 ciblée — 34-4 « transaction déjà comptabilisée » (#828) : la remédiation de la P3 (Sonnet, une lentille, lecture seule)

Tu es la lentille **unique** de la passe 4 de `bmad-code-review` de la story **34-4** de Kesh. C'est une **passe ciblée**
(`CLAUDE.md`, § *La passe ciblée*) : une seule question — **la remédiation de la P3 a-t-elle introduit un défaut, ou laissé
un finding de la P3 sans effet ?** Contexte frais, français soigné.

## Le périmètre : le seul commit de la remédiation P3

```sh
git -C /home/gcorbaz/devel/kesh-e344 show --stat 1005d74a
git -C /home/gcorbaz/devel/kesh-e344 show 1005d74a
```

Trois fichiers, **texte seulement** : `crates/kesh-i18n/locales/de-CH/messages.ftl`, `docs/manual/fr/user-manual.tex`, la fiche
`_bmad-output/implementation-artifacts/34-4-transaction-deja-comptabilisee.md` (annotation de l'AC-6). Les findings remédiés
sont dans `/home/gcorbaz/devel/kesh-gate-logs/e344-review-p3-ciblee.md` (C3-1, C3-3, C3-4 ; C3-2 est l'issue #1024, non
corrigée ici, et #1023 est le rejet sans confirmation).

## Axes

1. **C3-1** : l'annotation de l'AC-6 et la phrase du manuel (« *Continuer quand même* reprend le geste… si la ligne a quitté
   la liste… ne s'ouvrent pas, sans message (issue \#1024) ») disent-elles ce que fait `confirmGesture`
   (`frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte`) ? *Accepter* : le manuel dit « le lot entier » —
   est-ce vrai si une ligne cochée a quitté la liste (lis `onAccept`) ? Reste-t-il ailleurs dans le manuel, la fiche ou le
   CHANGELOG un « reprend le geste tel quel » faux (`/usr/bin/grep -rnF "tel quel"` sur ces fichiers) ?
2. **C3-4** : la phrase neuve sur ce qui fonde le signal (montant ; pour le second côté, transaction de sens contraire
   rapprochée sur un autre compte ; jamais la date ni le libellé) est-elle vraie contre `MATCHED_ELSEWHERE` et
   `ENTRY_NOT_REVERSED_NOR_SETTLEMENT` (`crates/kesh-db/src/repositories/reconciliation_link.rs`) ? Contredit-elle un autre
   passage du § `sec:deja-comptabilisee` ?
3. **C3-3** : l'allemand des trois clés (`…-already-booked-one`, `-other`, `-confirm-intro`) — grammaire, sens égal au fr-CH,
   aucune répétition, aucun `{ $count }` perdu, aucun sélecteur Fluent (`->`).
4. **LaTeX** : la phrase modifiée compile-t-elle à la lecture (accolades, `~`, `\#`, tirets) ? Le PDF n'est pas régénéré dans
   la branche : ne le contrôle pas, dis-le.

## ⛔ Ground-truth obligatoire

Toute affirmation de présence ou d'absence s'appuie sur `/usr/bin/grep -nF "<chaîne exacte>" <fichier>` (ou
`git -C /home/gcorbaz/devel/kesh-e344 show 1005d74a:<fichier> | /usr/bin/grep -nF …`), **dont tu copies la sortie**.
(`grep` est une fonction vers `ugrep` : utilise **`/usr/bin/grep`**.)

## Sévérités

HIGH (le manuel affirme un comportement que le code n'a pas, sur un geste comptable), MEDIUM (incohérence
code/manuel/fiche, texte allemand faux), LOW (rédaction).

## Interdits — lecture seule stricte

Aucune commande qui écrit, compile ou exécute : pas de `cargo`, `npm`, `npx`, `make`, `latexmk`, `docker`, `curl`, `sqlx`,
`mariadb` ; **pas de `scripts/prepare-release.sh`** ni d'aucun script de `scripts/` ni de `kesh-gate-logs/` ; pas de
`git checkout`/`switch`/`restore`, `git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase` ; aucune commande
`gh` qui écrit ; pas de `claude`, `pkill`, `killall`, `rm`, `sed -i`, `perl -pi`, `tee`, **aucune redirection `>` ni fichier
temporaire**. **Tu n'écris qu'un seul fichier** : ton rapport.

## Rapport

Écris-le dans `/home/gcorbaz/devel/kesh-gate-logs/e344-review-p4-ciblee.md` :

1. **Décompte** CRITICAL / HIGH / MEDIUM / LOW, recompté depuis ta liste.
2. **Findings** : identifiant (`C4-1`…), sévérité, fichier et ligne, sortie `grep -nF` copiée, correctif proposé.
3. **Axes exercés et non exercés** — liste explicite ; un « 0 finding » sans cette liste ne compte pas.
