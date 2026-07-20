/**
 * Re-exports publics de la feature `dunning` (rappels débiteurs — Story 21-4, #231).
 */

export type {
	DunningLevelResponse,
	CreateDunningLevelPayload,
	UpdateDunningLevelPayload,
	DunningSettingsResponse,
	UpdateDunningSettingsRequest,
} from './dunning.types';
export {
	listDunningLevels,
	createDunningLevel,
	updateDunningLevel,
	deleteDunningLevel,
	getDunningSettings,
	updateDunningSettings,
} from './dunning.api';
