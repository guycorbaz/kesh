# Prompt — validation P7, Story 32-7 (l'écran « Serveur MCP » et la section du manuel)

*Versionné le 2026-10-25. Passe **ciblée** (septième passe sur huit) : **une seule lentille, « regression hunter »**,
en contexte frais, **Sonnet**, braquée sur **le seul commit du retrait de la P6** — le commit qui suit `f1063a00` sur la
branche `story/32-7` (`git log --oneline -3`). Lecture seule stricte. La P5 a rendu 0/0/2/7, la P6 (Opus) 0/0/1/3 ; son
MEDIUM (P6-1) naissait de la remédiation P5 : la phrase du `keshwarning` prescrivait de déclarer « la passerelle »
comme mandataire. La remédiation a été un **retrait**, et P6-2 à P6-4 (LOW) ont été renvoyés au § *À recontrôler à
l'étape 0* sans être remédiés au corps de la fiche. ⚠️ Dès la P6, l'orchestrateur ne remédie plus lui-même. La boucle
ne se clôt que sur une passe sans finding au-dessus de LOW.*

Dépôt `/home/gcorbaz/devel/kesh-32-7`, branche `story/32-7`. Fiche :
`_bmad-output/implementation-artifacts/32-7-ecran-serveur-mcp.md` ; registre :
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-327.md` (non touché par ce commit : à le vérifier).
Rapport P6 : `/home/gcorbaz/devel/kesh-gate-logs/327-validate-p6.md` ; entrée « validation P6 » du Change Log. Diff :

```sh
git -C /home/gcorbaz/devel/kesh-32-7 diff f1063a00 HEAD -- \
  _bmad-output/implementation-artifacts/32-7-ecran-serveur-mcp.md \
  _bmad-output/implementation-artifacts/epic-32-choix-autonomes-327.md
```

Lis-le **en entier**, puis chaque site touché dans son contexte (`sed -n`, une trentaine de lignes autour). Code de la
32-6a en lecture seule : `/home/gcorbaz/devel/kesh-32-6/crates/kesh-api/src/mcp/reseau.rs` et la fiche
`/home/gcorbaz/devel/kesh-32-6/_bmad-output/implementation-artifacts/32-6-endpoint-mcp.md` (l. 1001-1008, 1326-1331).
Règles : `CLAUDE.md` (§ *La passe ciblée*, *Propagation post-patch*, *Recompter ses propres comptes rendus* — « recompter
n'est pas réécrire l'historique »).

**Décisions acquises** : P6-1 est remédié **par retrait** du second cas (la passerelle) à ses trois sites vivants ;
cela **rouvre la décision R1 = F1 de l'orchestrateur sur ce seul second cas** (déclaré au Change Log). Ne pas la
contester ; contrôler qu'elle est appliquée sans résidu. P6-2, P6-3, P6-4 sont **renvoyés à l'étape 0** : leur absence
du corps de la fiche n'est pas un défaut, leur inscription au § *À recontrôler* doit être exacte.

⚠️ **Outillage** : `grep` est une fonction qui lance `ugrep`, qui refuse les répétitions bornées larges. Employer
`command grep`, et des fenêtres de 30 au plus.

## La lentille — regression hunter sur le retrait de la P6

Cherche ce que **ce commit** a cassé, contredit ou affirmé sans source. Axes, tous à exercer :

1. **La phrase de garde neuve** (« à condition que seul Traefik soit déclaré », reprise de la fiche 32-6, l. 1007-1008)
   et la phrase du `keshwarning` après retrait : affirment-elles plus que leur source ? Sont-elles grammaticalement
   et logiquement cohérentes avec la phrase précédente (« il peut voir la passerelle »), avec Nginx et DSM, qui ne
   sont pas Traefik ? La condition « seul Traefik » est-elle fausse ou trompeuse pour la recette Nginx ?
2. **Résidus du second cas** : `command grep -nF 'passerelle'` et `command grep -nF '172.17.0.1'` sur la fiche et le
   registre, **sorties copiées** ; chaque site restant est-il descriptif, annoté, ou historique (entrées P4/P5) ? Un
   site vivant prescrit-il encore de déclarer la passerelle (AC-6 (1), AC-18 (4), *Ce que les gardes ne voient pas*,
   demande 11, mutations, captures) ? L'AC-6 (1) « peuvent paraître » est-elle intacte ?
3. **Le contrôle du PDF (AC-21) — axe obligatoire, PDF compris** : **rejoue** chacun des quinze motifs sur
   `admin-manual.pdf` et celui de `user-manual.pdf` :
   `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | command grep -cE '<motif>'`, **sortie
   copiée** ; chacun doit rendre 0 (aucun fichier de `docs/` n'est touché : `git diff --stat f1063a00 HEAD -- docs`).
   Les motifs sont listés à l'AC-21 de la fiche (l. 878-883 environ).
4. **Le § *À recontrôler à l'étape 0*** : les trois points P6-2, P6-3, P6-4 y sont-ils, avec la correction attendue
   de chacun ? Les bornes citées sont-elles exactes (`textes_coherents.rs`, fonction `aucun_renvoi_au_menu_reglages`,
   fin à la l. 186 ; `command grep -nE 'ipv4_address|subnet|ipam'` sur `docs/manual/fr/admin-manual.tex`) ? Le point
   G2 existant a-t-il été modifié de façon cohérente ?
5. **Historique** : une entrée passée du Change Log ou du registre a-t-elle été **réécrite** au lieu d'être annotée
   (`git diff f1063a00 HEAD` : les lignes `-` ne doivent toucher que des textes vivants) ? L'annotation de l'entrée P5
   est-elle ajoutée, non substituée ?
6. **Décomptes** de l'entrée P6, recomptés à la source : trend P5 0/0/2/7 → P6 0/0/1/3 contre le rapport P6 ; 25 AC,
   9 tâches, 26 mutations, 35 reports, 27 choix, 29 captures, 21 faits, motifs 15 + 1.
7. **Références croisées** : le renvoi à « P6-1 », la mention de la fiche 32-6 (l. 1006-1008 ou 1007-1008), la demande 11
   récrite, l'en-tête de la fiche.
8. **Ce que le commit omet** : un site qui dit encore l'ancienne phrase sous une autre forme (« port publié »,
   « publication de port en espace utilisateur » comme adresse à déclarer), à l'écran, au manuel, aux captures, au
   registre.

## Ce que tu rends

Findings `P7-1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, site du dépôt, preuve (commande et sortie
copiée — `grep -nF` ou `command grep -nF` pour toute présence ou absence), correction proposée — de préférence un
**retrait**. Pour chaque finding, dis s'il **naît du retrait de la P6** (oui / non). Un bilan d'en-tête **cohérent avec
le corps**. ⛔ **La liste des axes exercés ET non exercés** (1 à 8) — sans elle, « 0 finding » ne compte pas. Ce que la
fiche **prescrit** au développement n'existe pas encore : son absence n'est pas un défaut. Rapport écrit **d'un seul
jet**, par l'outil d'écriture de fichiers : `/home/gcorbaz/devel/kesh-gate-logs/327-validate-p7.md`. Dernier message :
chemin, bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport, **aucun fichier temporaire**, rien dans le scratchpad. **Aucune
redirection** (`>`, `>>`, `2>`, `tee`, `<<`), pas même vers un fichier jetable — un fichier créé par erreur se déclare
au rapport, il ne se supprime pas. **Aucun interpréteur** : ni `python3`, `python`, `perl`, `ruby`, `node`, `deno`, ni
**`awk`**, ni `sed` autrement qu'en `sed -n`. **Aucun `gh` qui écrit** (`pr`, `issue create|edit|close|comment`,
`api` en écriture…) ; seuls `gh issue view 738 --json title,body,comments` et `gh issue view 758 --json title,body`
sont permis. Aucun `git checkout`/`switch`/`commit`/`add`/`stash`/`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`. Aucune
commande qui écrit, compile ou exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, **`scripts/test-fast.sh`**,
**`scripts/mem-guard.sh`**), `make`, `latexmk`, `cargo`, `rustup`, `npm`, `npx`, Playwright, `docker`, `curl`,
`mariadb`/`mysql`, `sed -i`. `pdftotext` vers la sortie standard seulement (`-` en sortie). Rien de `~/kesh-test/`,
aucune donnée réelle. Le dépôt principal `/home/gcorbaz/devel/kesh` est hors de ton périmètre ; les worktrees voisins
ne se lisent qu'en lecture. Autorisés : lecture de fichiers, `grep`/`command grep`, `sed -n`,
`git log|show|diff|blame|merge-base`, `pdftotext … -`, les deux `gh issue view` ci-dessus, et les filtres `tr`, `sort`,
`cut`, `head`, `tail`, `wc`, `uniq` vers la sortie standard. Si une commande est refusée, arrête-toi et dis-le au
rapport.

---

## Annexe — le prompt du retrait de la P6, recopié pour que la boucle soit rejouable

> Tu es l'agent de spécification de la Story 32-7 de Kesh. Tu travailles **uniquement** dans le worktree `/home/gcorbaz/devel/kesh-32-7`, sur sa branche, et tu écris en français.
>
> Applique en entier les consignes de `/home/gcorbaz/.claude/projects/-home-gcorbaz-devel-kesh/memory/consignes-agents-epic15.md`, en particulier :
> - chemins absolus ;
> - dépôt principal en lecture seule ;
> - pas de push, pas de `gh` qui écrit, pas de prepare-release ;
> - ni cargo, ni npm, ni docker, ni curl ; la lecture et `pdftotext` vers la sortie standard sont permis ;
> - si une commande est refusée, tu t'arrêtes ;
> - n'ouvre jamais `~/kesh-test/`.
>
> **Contexte.** La validation P6 (rapport `/home/gcorbaz/devel/kesh-gate-logs/327-validate-p6.md`, à lire en entier) a rendu 0/0/1/3. Nous sommes en passe 6 sur 8. La règle du projet veut qu'à ce stade on **retire**, sans réécrire ni enrichir.
>
> **Tâche, strictement limitée :**
>
> 1. **P6-1 (MEDIUM), par retrait.** Supprime le second cas, « derrière une publication de port en espace utilisateur, la passerelle », à ses trois sites (l. ≈769, ≈1036-1037, ≈1278-1280 ; vérifier chacun par `grep -nF`). La phrase descriptive et la demande 11 restent. Ajoute une seule phrase de garde, prise mot pour mot de la fiche 32-6 si elle existe là-bas, sinon n'ajoute rien. Déclare au Change Log que P6-1 rouvre la décision R1 = F1 de l'orchestrateur sur ce seul second cas.
> 2. **Les trois LOW (P6-2, P6-3, P6-4).** Ne les remédie pas dans le corps de la fiche. Ajoute-les au § *À recontrôler à l'étape 0*, chacun avec la correction attendue : P6-2 (borne de G2 jusqu'à la l. 186 ; ancienne borne `153-166` à la l. ≈1386) ; P6-3 (l'adresse de Traefik n'est pas fixe d'une recréation à l'autre ; à dire au manuel par la 32-7 à son développement, ou à éprouver par la 32-11) ; P6-4 (la citation de la l. 440 doit être reprise).
> 3. **Après le retrait :** greppe `passerelle` et `172.17.0.1` sur la fiche ; rejoue les quinze motifs de l'AC-21 sur `admin-manual.pdf` et cite les sorties ; ne réécris aucune entrée passée ; ajoute une entrée P6 neuve au Change Log (trend P5 0/0/2/7 → P6 0/0/1/3, remédiation par retrait, LOW renvoyés à l'étape 0).
>
> **Livrables :** un commit `refs #723`, sans mot-clé de fermeture ; le prompt de la P7 (`_bmad-output/implementation-artifacts/327-validate-prompt-p7.md`), passe ciblée, une lentille, regression hunter, **Sonnet**, sur le seul commit de ce retrait, rapport dans `/home/gcorbaz/devel/kesh-gate-logs/327-validate-p7.md`, mêmes exigences et mêmes interdits que la P6, ce prompt recopié en annexe ; un rapport final bref.
