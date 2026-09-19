import { msg } from '@lingui/core/macro';

/** Extraction does not read `.svelte`; ids are unchanged from the React components. */
export const projectMessages = {
  // Page
  noOrganization: msg`No organization selected`,
  noOrganizationHint: msg`Select an organization from the sidebar to view its projects`,
  loadError: msg`Unable to load projects`,

  // Header
  project: msg`Project`,
  projects: msg`Projects`,
  newProject: msg`New project`,
  createDenied: msg`You don't have permission to create projects.`,
  deleteDenied: msg`You don't have permission to delete projects.`,

  // Card
  edit: msg`Edit`,
  select: msg`Select`,
  noDescription: msg`No description`,

  // Dialogs
  projectName: msg`Project name`,
  projectNamePlaceholder: msg`e.g., My project`,
  description: msg`Description`,
  descriptionPlaceholder: msg`e.g., A short description of the project`,
  addDescription: msg`Add a description...`,
  createTitle: msg`Create a new project`,
  createDescription: msg`Enter a name for your new project and select the organization it belongs to.`,
  createSubmit: msg`Create Project`,
  editTitle: msg`Edit project`,
  editDescription: msg`Update the project name and description.`,
  save: msg`Save`,
  saving: msg`Saving...`,
};
