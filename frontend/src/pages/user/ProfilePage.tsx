import UserProfile from "../../components/user/UserProfile";
import MainPage from "../main/MainPage";

export default function ProfilePage() {
  return (
    <MainPage>
      <UserProfile
        username="username"
        hostname="example.com"
        nickname="user dayo"
        posts={[]}
      />
    </MainPage>
  );
}
