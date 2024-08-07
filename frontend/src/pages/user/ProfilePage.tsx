import useSWR from "swr";
import UserProfile from "../../components/user/UserProfile";
import MainPage from "../main/MainPage";
import { useSelector } from "react-redux";
import { selectAuthorization } from "../../stores/authSlice";
import { authedFetcher } from "../../hooks";
import { useParams } from "react-router-dom";

interface UserResponse {
  id: string;
  username: string;
  host: string | null;
  nickname: string;
  bio: string;
  is_following_you?: boolean;
  is_followed_by_you?: boolean;
}

export default function ProfilePage() {
  const { userId: id } = useParams<{
    userId: string;
  }>();

  const authorization = useSelector(selectAuthorization);
  const { data, error, isLoading } = useSWR(
    [authorization, `/user/${id}`],
    authedFetcher<UserResponse>,
    {
      refreshInterval: 5000,
    }
  );

  if (error) {
    return "user fetch error";
  }

  if (isLoading || !data) {
    return "Loading...";
  }

  return (
    <MainPage>
      <UserProfile
        username={data.username}
        hostname={data.host ?? undefined}
        nickname={data.nickname}
        posts={[]}
        is_followed_by_you={data.is_followed_by_you}
        is_following_you={data.is_following_you}
      />
    </MainPage>
  );
}
