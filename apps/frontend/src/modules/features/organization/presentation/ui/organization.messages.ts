import { msg } from '@lingui/core/macro';

/** Extraction does not read `.svelte`; ids are unchanged from the React components. */
export const organizationMessages = {
  // List rows
  members: msg`Members`,
  edit: msg`Edit`,
  delete: msg`Delete`,

  // Add dialog
  createTitle: msg`Create a new organization`,
  createDescription: msg`Enter a name and description for your new organization. You can change these later in settings.`,
  createSubmit: msg`Create Organization`,

  // Edit dialog
  editTitle: msg`Edit organization`,
  editDescription: msg`Update the organization name and description.`,
  save: msg`Save`,
  saving: msg`Saving...`,
  organizationName: msg`Organization name`,
  description: msg`Description`,
  addDescription: msg`Add a description...`,
};
