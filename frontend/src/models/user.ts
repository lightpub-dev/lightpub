export interface UserResponse {
  id: string;
  username: string;
  host: string | null;
  nickname: string;
  bio: string;
  is_following_you?: boolean;
  is_followed_by_you?: boolean;
}
