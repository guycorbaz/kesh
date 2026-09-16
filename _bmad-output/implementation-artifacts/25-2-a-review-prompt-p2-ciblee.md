# Story 25-2-a — prompt de la passe 2, **ciblée**

**Modèle** : Haiku 4.5 (cycle Sonnet → Haiku → Opus ; la passe 1 était Sonnet).
**Lentille unique** : chasseur de régression sur la remédiation.
**Périmètre** : le **seul** commit `d32a0244`, remédiation de la passe 1.
**Contexte** : frais.

## Pourquoi une passe ciblée, et pourquoi la boucle n'est pas close

La passe 1 a rendu **0 CRITICAL, 0 HIGH, 1 MEDIUM, 4 LOW** : la sévérité est retombée à zéro
au-dessus de LOW une fois le MEDIUM remédié. C'est le cas que la § *« La passe ciblée »* du
`CLAUDE.md` prévoit — *ce qu'il reste à relire n'est plus la story, c'est la dernière
remédiation.*

⛔ **Mais la boucle ne peut pas être close ici** : le critère de clôture est que la remédiation ne
touche **aucune ligne de code de production**, et celle-ci en touche — les trois lignes de
validation de `readRetypeImpact`. D'où cette passe.

**Le motif qui la justifie est mesuré, pas supposé** : sur l'Epic 23, **sept des huit passes** ont
trouvé une régression du patch précédent et **aucune** un défaut de la conception d'origine.

## Diff à relire — aplati, en un seul morceau

```sh
cd /home/gcorbaz/devel/kesh
git show d32a0244
```

⚠️ **Un diff unique et aplati, jamais une séquence de commits.** Haiku 4.5 indexe mal les lignes
d'un `git show A B` quand le second commit retouche des hunks du premier : il cherche la ligne *n*
et y voit le contenu d'avant. Quatre CRITICAL/HIGH hallucinés ont déjà été réfutés dans ce dépôt
par cette voie.

## Ce que la remédiation prétend faire

1. `frontend/src/lib/features/accounts/accounts.types.ts` — `readRetypeImpact` ne faisait échouer la
   validation que sur `entryCount` ; `closedFiscalYears` non-tableau devenait `[]`, `fromType` et
   `toType` manquants devenaient `''`. **Les quatre champs invalident désormais.**
2. La fiche de story : `Status` `ready-for-dev` → `review` ; deux angles morts inventoriés ; une
   référence périmée à la *25-2-b* corrigée en *25-2-c*.

## Axes — et rien d'autre

1. **La régression.** Le durcissement de `readRetypeImpact` casse-t-il un appelant ? Un `details`
   légitime du serveur peut-il désormais être **rejeté à tort** ? Compare le contrat réel émis par
   `crates/kesh-api/src/errors.rs` (le bloc `AccountHasEntries`) au contrat désormais exigé côté
   client. Un champ que le serveur n'enverrait pas ferait maintenant retomber l'écran sur le message
   générique **au lieu** d'ouvrir l'avertissement : est-ce possible ?
2. **Les tests suivent-ils ?** Les cinq tests de `accounts-page.test.ts` exercent-ils encore ce
   qu'ils prétendent, ou l'un d'eux passe-t-il désormais **par vacuité** ? En particulier celui du
   `details` illisible : invalide-t-il pour la raison qu'il annonce, ou pour une autre ?
3. **Les déclarations.** La fiche affirme des chiffres et des résultats. Ceux que ce commit touche
   sont-ils exacts ? ⚠️ **Recompte depuis la source**, ne relis pas sur parole.
4. **Ce que la remédiation aurait dû toucher et n'a pas touché.** Un symptôme corrigé à un endroit
   survit-il ailleurs ? Le commentaire de `readRetypeImpact` décrit-il exactement ce que le code
   fait — ni plus, ni moins ? *C'est ce défaut-là que la passe 1 a trouvé : un commentaire plus
   affirmatif que son code.*

## ⛔ Interdits absolus

- **N'écris AUCUN fichier.** Pas d'`Edit`, pas de `Write`, pas de `git commit`, pas de
  `git checkout`, pas de `git stash`.
- **N'exécute AUCUN script mutant**, nommément : `scripts/prepare-release.sh`,
  `scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`, ni aucun `make`. *Interdire l'écriture
  ne suffit pas : une passe antérieure a bumpé les dix crates du workspace en lançant
  `prepare-release.sh` malgré la consigne.* Lancer des tests en lecture est permis.

## Méthode imposée

**Tout finding CRITICAL ou HIGH doit porter sa preuve au sol** : `grep -nF "<chaîne exacte>"
<fichier>`, le `-F` étant obligatoire — le TypeScript est plein de métacaractères. Un CRITICAL sans
cette preuve sera écarté sans examen.

## Format du rapport

1. **Les axes réellement exercés, et ceux qui ne l'ont pas été.** Liste obligatoire. Un « 0 finding »
   qui ne s'y adosse pas **ne compte pas comme passe**, et l'orchestrateur reprendra lui-même tout
   axe déclaré non exercé.
2. Les findings, du plus grave au moins grave, avec fichier, ligne et scénario d'échec concret.
3. Ce qui a été vérifié et trouvé correct, une ligne chacun.

N'invente rien. Si tu n'as pas vérifié, dis-le.
