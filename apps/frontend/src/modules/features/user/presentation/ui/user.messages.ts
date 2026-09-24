import { msg } from '@lingui/core/macro';

export const userMessages = {
  user: msg`User`,
  users: msg`Users`,
  newUser: msg`New user`,
  createDenied: msg`You don't have permission to create users.`,
  deleteDenied: msg`You don't have permission to delete users.`,
  loadError: msg`Error loading users`,

  createTitle: msg`Create a new user`,
  createDescription: msg`Enter a username and password for the new user.`,
  createSubmit: msg`Create User`,
  username: msg`Username`,
  usernamePlaceholder: msg`e.g., john.doe`,
  password: msg`Password`,
  passwordPlaceholder: msg`Enter a password`,

  createdAt: msg`Created at`,
  actions: msg`Actions`,
  view: msg`View`,

  userSettings: msg`User settings`,
  organizations: msg`Organizations: `,
  userInformation: msg`User information`,
  manageAccount: msg`Manage your account details.`,
  informationUnavailable: msg`User information not available`,
  loadingInformation: msg`Loading user information...`,
  informationError: msg`Error loading user information`,
  activeAccount: msg`Active account`,
  userId: msg`User ID`,
  save: msg`Save`,
};
