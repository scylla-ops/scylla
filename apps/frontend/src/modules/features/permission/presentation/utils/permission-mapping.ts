import {
  Permission,
  PermissionScope,
} from '@/modules/features/permission/domain/structs/permission.struct.ts';

export const ALL_SCOPES: PermissionScope[] = [
  PermissionScope.SYSTEM,
  PermissionScope.ORGANIZATION,
  PermissionScope.PROJECT,
];

export const scopeName = (scope: PermissionScope): string => {
  switch (scope) {
    case PermissionScope.SYSTEM:
      return 'System';
    case PermissionScope.ORGANIZATION:
      return 'Organization';
    case PermissionScope.PROJECT:
      return 'Project';
    default:
      return 'Unknown';
  }
};

export interface PermissionDefinition {
  id: Permission;
  label: string;
  scope: PermissionScope;
  /** Permissions requises pour que celle-ci ait du sens */
  dependsOn?: Permission[];
}

export const PERMISSION_DEFINITIONS: Record<Permission, PermissionDefinition> = {
  // --- USERS & SYSTEM ---
  [Permission.READ_USER]: {
    id: Permission.READ_USER,
    label: 'Voir les utilisateurs',
    scope: PermissionScope.SYSTEM,
  },
  [Permission.CREATE_USER]: {
    id: Permission.CREATE_USER,
    label: 'Créer un utilisateur',
    scope: PermissionScope.SYSTEM,
    dependsOn: [Permission.READ_USER],
  },
  [Permission.DELETE_USER]: {
    id: Permission.DELETE_USER,
    label: 'Supprimer un utilisateur',
    scope: PermissionScope.SYSTEM,
    dependsOn: [Permission.READ_USER],
  },

  // --- ORGANIZATIONS ---
  [Permission.READ_ORGANIZATION]: {
    id: Permission.READ_ORGANIZATION,
    label: "Voir l'organisation",
    scope: PermissionScope.ORGANIZATION,
  },
  [Permission.UPDATE_ORGANIZATION]: {
    id: Permission.UPDATE_ORGANIZATION,
    label: "Modifier l'organisation",
    scope: PermissionScope.ORGANIZATION,
    dependsOn: [Permission.READ_ORGANIZATION],
  },
  [Permission.DELETE_ORGANIZATION]: {
    id: Permission.DELETE_ORGANIZATION,
    label: "Supprimer l'organisation",
    scope: PermissionScope.ORGANIZATION,
    dependsOn: [Permission.READ_ORGANIZATION],
  },
  [Permission.MANAGE_ORG_GRANTS]: {
    id: Permission.MANAGE_ORG_GRANTS,
    label: "Gérer les membres et rôles d'organisation",
    scope: PermissionScope.ORGANIZATION,
    dependsOn: [Permission.READ_ORGANIZATION],
  },

  // --- PROJECTS ---
  [Permission.READ_PROJECT]: {
    id: Permission.READ_PROJECT,
    label: 'Voir les projets',
    scope: PermissionScope.PROJECT,
    // Nécessite de pouvoir lire l'organisation parente
    dependsOn: [Permission.READ_ORGANIZATION],
  },
  [Permission.CREATE_PROJECT]: {
    id: Permission.CREATE_PROJECT,
    label: 'Créer un projet',
    scope: PermissionScope.ORGANIZATION,
    dependsOn: [Permission.READ_ORGANIZATION],
  },
  [Permission.UPDATE_PROJECT]: {
    id: Permission.UPDATE_PROJECT,
    label: 'Modifier le projet',
    scope: PermissionScope.PROJECT,
    dependsOn: [Permission.READ_PROJECT],
  },
  [Permission.DELETE_PROJECT]: {
    id: Permission.DELETE_PROJECT,
    label: 'Supprimer le projet',
    scope: PermissionScope.PROJECT,
    dependsOn: [Permission.READ_PROJECT],
  },

  // --- PIPELINES ---
  [Permission.LIST_PIPELINES_BY_PROJECT]: {
    id: Permission.LIST_PIPELINES_BY_PROJECT,
    label: 'Lister les pipelines du projet',
    scope: PermissionScope.PROJECT,
    // Dépend explicitement de la lecture du projet
    dependsOn: [Permission.READ_PROJECT],
  },
  [Permission.READ_PIPELINE]: {
    id: Permission.READ_PIPELINE,
    label: 'Voir le détail des pipelines',
    scope: PermissionScope.PROJECT,
    dependsOn: [Permission.LIST_PIPELINES_BY_PROJECT],
  },
  [Permission.CREATE_PIPELINE]: {
    id: Permission.CREATE_PIPELINE,
    label: 'Créer une pipeline',
    scope: PermissionScope.PROJECT,
    dependsOn: [Permission.READ_PIPELINE],
  },
  [Permission.RUN_PIPELINE]: {
    id: Permission.RUN_PIPELINE,
    label: 'Lancer une pipeline',
    scope: PermissionScope.PROJECT,
    dependsOn: [Permission.READ_PIPELINE],
  },

  // ... Compléter avec le reste si nécessaire
} as Record<Permission, PermissionDefinition>;

const SCOPE_HIERARCHY: Record<PermissionScope, PermissionScope[]> = {
  [PermissionScope.SYSTEM]: [
    PermissionScope.SYSTEM,
    PermissionScope.ORGANIZATION,
    PermissionScope.PROJECT,
  ],
  [PermissionScope.ORGANIZATION]: [PermissionScope.ORGANIZATION, PermissionScope.PROJECT],
  [PermissionScope.PROJECT]: [PermissionScope.PROJECT],
  [PermissionScope.UNSPECIFIED]: [],
};

/**
 * Renvoie les DEFINITIONS complètes des permissions valides pour un scope donné
 */
export function getPermissionDefinitionsForScope(scope: PermissionScope): PermissionDefinition[] {
  const allowedScopes = SCOPE_HIERARCHY[scope] ?? [];
  return Object.values(PERMISSION_DEFINITIONS).filter(def => allowedScopes.includes(def.scope));
}

/**
 * Renvoie uniquement les ENUMS (ids) des permissions valides pour un scope donné
 */
export function getPermissionsForScope(scope: PermissionScope): Permission[] {
  return getPermissionDefinitionsForScope(scope).map(def => def.id);
}

export function getPermissionLabel(permission: Permission): string {
  const perm = PERMISSION_DEFINITIONS[permission];
  return perm ? perm.label : 'Unknown Permission';
}
