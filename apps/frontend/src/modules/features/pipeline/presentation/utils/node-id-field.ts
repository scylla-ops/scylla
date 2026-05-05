/**
 * JSON field name used for the node identifier in the pipeline script.
 * - Create uses "nodeId" (current backend contract for creation)
 * - Update uses "id" (current backend contract for update)
 *
 * Change these values when the backend unifies the field name.
 */
export const NODE_ID_FIELD_CREATE = 'nodeId';
export const NODE_ID_FIELD_UPDATE = 'id';

