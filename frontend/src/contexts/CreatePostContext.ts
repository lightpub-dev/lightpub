import { createContext } from "react";

export type CreatePostContextType = {
  showCreatePost: (options: { reply_to_id?: string }) => void;
};

export const CreatePostContext = createContext<CreatePostContextType | null>(
  null
);
