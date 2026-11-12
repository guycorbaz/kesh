# Revue de code P3 ciblée — 34-5 (#951), une lentille (Haiku, lecture seule)

Tu es la **seule** lentille de la passe 3 de `bmad-code-review` de la story **34-5-reglement-idempotent** de Kesh
(issue #951 : un second envoi du même règlement de facture client, sans relecture, écrivait un second règlement).
Modèle : **Haiku**. Contexte frais. Travaille en français soigné.

C'est une **passe ciblée** : elle ne relit pas la story, seulement **le commit de remédiation de la P2**. La P2 complète
(Sonnet, trois lentilles) a rendu 0 finding au-dessus de LOW et 8 LOW ; ce commit les remédie. Ta question est unique :
**ce commit a-t-il introduit un défaut ?**

## Le périmètre

Le commit de remédiation de la P2 est le dernier commit de code de la branche ; trouve-le ainsi :

```sh
git -C /home/gcorbaz/devel/kesh-e345 log --oneline -6
git -C /home/gcorbaz/devel/kesh-e345 show --stat <commit « remédiation de la revue de code P2 »>
git -C /home/gcorbaz/devel/kesh-e345 show <ce commit>
```

Lis-le **comme un seul diff** (`git show <commit>`). Fichiers attendus :
- `frontend/src/routes/(app)/invoices/[id]/+page.svelte` et `frontend/src/routes/(app)/invoices/due-dates/+page.svelte` :
  l'avis de conflit dit « Relecture en cours… » tant que `settleRereading` est vrai, puis « n'a pas pu être relue » si la
  relecture a échoué (B2-1) ;
- les quatre `crates/kesh-i18n/locales/*/messages.ftl` : deux clés neuves (`invoice-settle-conflict-rereading`,
  `due-dates-settle-conflict-rereading`) ; en de-CH, quatre phrases « bleibt nicht verfügbar » récrites (E2-2) ;
- `crates/kesh-api/src/routes/invoices.rs` : un doc-comment recoupé (B2-2) ;
- `frontend/src/lib/shared/i18n-keys.test.ts` (borne `sitesTotal` 2928 → 2930) ;
- les deux Vitest des pages (A1-2 récrit, test neuf de la fiche).

Contexte, à lire seulement pour juger le diff : la fiche
`/home/gcorbaz/devel/kesh-e345/_bmad-output/implementation-artifacts/34-5-reglement-idempotent.md` (entrée « revue de
code P2 » du Change Log) et les rapports `/home/gcorbaz/devel/kesh-gate-logs/e345-review-p2-{B,E,A}.md`.

## Axes de la lentille

1. **Le texte suit-il l'état ?** Pour chaque état (`conflict`, `conflict-blocked` avec `settleRereading` vrai, puis faux),
   l'avis affiché est-il le bon, fiche et échéancier ? *Relire* n'apparaît-il qu'une fois la relecture échouée ? Une
   relecture qui réussit passe-t-elle bien à l'avis « relue » (y compris sur l'échéancier, où `load()` lève l'état) ?
2. **i18n** : les deux clés neuves existent-elles aux quatre locales, avec `{ $number }` pour l'échéancier ? Les replis
   Svelte égalent-ils les textes `fr-CH` caractère pour caractère ? Les quatre phrases de-CH récrites disent-elles
   « indisponible jusqu'à la relecture » sans contresens ? La borne `sitesTotal` égale-t-elle +2 (deux appels neufs) ?
3. **Tests** : le Vitest A1-2 récrit et le Vitest neuf de la fiche prouvent-ils le texte « en cours » pendant une
   relecture **réellement en attente** (promesse non résolue), puis l'avis « relue » ? Un test passe-t-il à vide ?
4. **Mutations** de ce commit (`/home/gcorbaz/devel/kesh-gate-logs/e345/mut/M12.log`, `M12e.log`, et le rejeu
   `M1.log` à `M6e.log`, `M11.log`) : appliquées, rouges pour la bonne raison, restaurées ?
5. **Rien d'autre n'a bougé** : le diff ne touche-t-il aucune ligne hors de ce que la P2 demandait ?

## ⛔ Ground-truth obligatoire

Toute affirmation de présence ou d'absence d'un texte s'appuie sur `/usr/bin/grep -nF "<chaîne exacte>" <fichier>` (ou
`git -C /home/gcorbaz/devel/kesh-e345 show HEAD:<fichier> | /usr/bin/grep -nF …`), **dont tu copies la sortie**. Pour un
bloc, `-A5`. (`grep` est une fonction vers `ugrep` : utilise **`/usr/bin/grep`**.) N'affirme pas qu'un patch manque sans
cette sortie : lis le fichier tel qu'il est à `HEAD`, non les numéros de ligne du diff.

## Sévérités

CRITICAL (un même envoi écrit deux fois, perte de données), HIGH (texte faux pour un état, geste rouvert sur des montants
périmés, test qui ne prouve rien), MEDIUM (incohérence texte/locale/repli, mutation tuée pour une mauvaise raison), LOW
(rédaction).

## Interdits — lecture seule stricte

Aucune commande qui écrit, compile ou exécute : pas de `cargo`, `npm`, `npx`, `make`, `docker`, `curl`, `sqlx`,
`mariadb` ; **pas de `scripts/prepare-release.sh`** ni d'aucun script de `scripts/` ; pas de `git checkout` / `switch`,
`git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase` ; aucune commande `gh` qui écrit ; pas de
`claude`, `pkill`, `killall`, `rm`, `sed -i`, `perl -pi`, `tee`, **aucune redirection `>` ni fichier temporaire**. **Tu
n'écris qu'un seul fichier** : ton rapport.

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/e345-review-p3-ciblee.md` (jamais sous `target/`) :

1. **Décompte** CRITICAL / HIGH / MEDIUM / LOW, recompté depuis ta liste.
2. **Findings** : identifiant (`C3-1`…), sévérité, fichier et ligne, sortie `grep -nF` copiée, correctif proposé.
3. **Axes exercés et non exercés** — liste explicite ; un « 0 finding » sans cette liste ne compte pas.
