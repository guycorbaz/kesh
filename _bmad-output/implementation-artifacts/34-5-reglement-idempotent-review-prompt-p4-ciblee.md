# Revue de code P4 ciblée — 34-5 (#951), une lentille (Sonnet, lecture seule)

Tu es la **seule** lentille de la passe 4 de `bmad-code-review` de la story **34-5-reglement-idempotent** de Kesh
(issue #951 : un second envoi du même règlement de facture client, sans relecture, écrivait un second règlement).
Modèle : **Sonnet**. Contexte frais. Travaille en français soigné.

C'est une **passe ciblée** : elle ne relit que **le commit de remédiation de la passe ciblée P3**, `59a90ed7`. La P3
(Haiku) a rendu 1 MEDIUM et 1 LOW, que ce commit remédie. Ta question est unique : **ce commit corrige-t-il ce qu'il
prétend corriger, et a-t-il introduit un défaut ?**

## Le périmètre

```sh
git -C /home/gcorbaz/devel/kesh-e345 show --stat 59a90ed7
git -C /home/gcorbaz/devel/kesh-e345 show 59a90ed7
```

Fichiers attendus :
- `frontend/src/routes/(app)/invoices/due-dates/+page.svelte` : `rereadAfterSettle` ne remet plus `settleRereading` à
  faux ; c'est `load()`, dans son `finally`, qui le fait à la fin du **chargement le plus récent**
  (`seq === loadSeq`) — C3-1 ;
- `frontend/src/routes/(app)/invoices/due-dates/due-dates-page.test.ts` : un Vitest neuf, une relecture supplantée par
  un filtre ;
- `crates/kesh-i18n/locales/de-CH/messages.ftl` : deux espaces retirées avant « … » — C3-2.

Contexte, à lire seulement pour juger le diff : le rapport de la P3
`/home/gcorbaz/devel/kesh-gate-logs/e345-review-p3-ciblee.md`, l'entrée de la P3 au Change Log de la fiche
`/home/gcorbaz/devel/kesh-e345/_bmad-output/implementation-artifacts/34-5-reglement-idempotent.md`, et le choix
C-e345-10 (`e34-choix-autonomes-e345.md` : tout chargement réussi lève l'état bloquant).

## Axes de la lentille

1. **C3-1 est-il fermé ?** Suis `settleRereading` dans tous les chemins de l'échéancier :
   - relecture seule, réussie, en échec ;
   - relecture supplantée par un filtre qui réussit, qui échoue ;
   - plusieurs filtres successifs ;
   - un chargement ordinaire sans relecture (page, tri, recherche) ;
   - l'issue inconnue de la 33-6 (`blocked`), qui passe par la même fonction.

   Pendant toute relecture en vol, l'avis dit-il « en cours », et *Relire* est-il caché ? À l'issue du dernier
   chargement, l'état est-il juste ?
2. **Régression** : `loading` et `settleRereading` peuvent-ils rester bloqués à vrai, par exemple sur un chargement
   dont le `seq` ne redevient jamais le plus récent ? Un chargement ordinaire qui finit pendant une relecture peut-il
   lever `settleRereading` trop tôt ? Le `finally` couvre-t-il le chemin d'exception ?
3. **Le test prouve-t-il** : la promesse de la relecture est-elle vraiment résolue **avant** celle du filtre ? L'attente
   intermédiaire (`setTimeout(0)`) suffit-elle pour que l'assertion « en cours » porte sur l'état d'après le retour de
   la relecture ? Le test passerait-il à vide ?
4. **Mutation M13** (`/home/gcorbaz/devel/kesh-gate-logs/e345/mut/M13.log` ; elle remet `settleRereading = false`
   juste après le `load()` de la relecture) : appliquée, rouge pour la bonne raison, restaurée ? Les mutations rejouées
   sur ce commit (`M4e`, `M5`, `M6e`, `M9`, `M12e` ; `mut/campagne-p3.log`) sont-elles toujours tuées ?
5. **de-CH** : les deux lignes suivent-elles maintenant l'usage du fichier (« … » collé) ?
6. **Rien d'autre n'a bougé** hors de ces trois fichiers.

## ⛔ Ground-truth obligatoire

Toute affirmation de présence ou d'absence d'un texte s'appuie sur `/usr/bin/grep -nF "<chaîne exacte>" <fichier>` (ou
`git -C /home/gcorbaz/devel/kesh-e345 show 59a90ed7:<fichier> | /usr/bin/grep -nF …`), **dont tu copies la sortie**.
Pour un bloc, `-A5`. (`grep` est une fonction vers `ugrep` : utilise **`/usr/bin/grep`**.)

## Sévérités

CRITICAL (un même envoi écrit deux fois, perte de données), HIGH (geste rouvert sur des montants périmés, état bloqué
sans issue, test qui ne prouve rien), MEDIUM (texte faux pour un état, mutation tuée pour une mauvaise raison), LOW
(rédaction).

## Interdits — lecture seule stricte

Aucune commande qui écrit, compile ou exécute : pas de `cargo`, `npm`, `npx`, `make`, `docker`, `curl`, `sqlx`,
`mariadb` ; **pas de `scripts/prepare-release.sh`** ni d'aucun script de `scripts/` ; pas de `git checkout` / `switch`,
`git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase` ; aucune commande `gh` qui écrit ; pas de
`claude`, `pkill`, `killall`, `rm`, `sed -i`, `perl -pi`, `tee`, **aucune redirection `>` ni fichier temporaire**. **Tu
n'écris qu'un seul fichier** : ton rapport.

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/e345-review-p4-ciblee.md` (jamais sous `target/`) :

1. **Décompte** CRITICAL / HIGH / MEDIUM / LOW, recompté depuis ta liste.
2. **Findings** : identifiant (`C4-1`…), sévérité, fichier et ligne, sortie `grep -nF` copiée, correctif proposé.
3. **Axes exercés et non exercés** — liste explicite ; un « 0 finding » sans cette liste ne compte pas.
