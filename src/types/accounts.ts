export interface MinecraftAccount {
  id: string;
  username: string;
  uuid: string;
  access_token: string;
  refresh_token: string;
  expires_at: number;
  skin_url?: string;
  cape_url?: string;
  type?: 'microsoft' | 'offline';
  isActive?: boolean;
  lastUsed?: Date;
}

export interface OAuthSession {
  csrf_token: string;
  pkce_verifier: string;
  auth_url: string;
}