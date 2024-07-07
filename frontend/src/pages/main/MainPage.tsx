import NewPostDialog from "../../components/post/NewPostDialog";
import Sidebar from "../../components/sidebar/Sidebar";

function MainPage() {
  return (
    <>
      <div className={"flex"}>
        <Sidebar
          children={
            <div className={"flex flex-col"}>
              <h1>Content</h1>
              <NewPostDialog />
            </div>
          }
        />
      </div>
    </>
  );
}

export default MainPage;
