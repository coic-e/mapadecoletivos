import {
  createBrowserRouter,
  Navigate,
  createRoutesFromElements
} from "react-router-dom";
import {
  Landing,
  Collective,
  RootError,
  CollectivesMap,
  CreateCollective,
} from "../pages";

/**
 * Application routes
 * https://reactrouter.com/en/main/routers/create-browser-router
 */
export const router = createBrowserRouter([
  {
    path: "/",
    element: <Landing />,
    errorElement: <RootError />,
  },
  {
    path: "/raves",
    element: <CollectivesMap />,
    errorElement: <RootError />,
  },
  {
    path: "/raves/create",
    element: <CreateCollective />,
    errorElement: <RootError />,
  },
  {
    path: "/raves/:id",
    element: <Collective />,
    errorElement: <RootError />,
  },
  {
    path: "*",
    element: <Navigate to="/" />,
  },
]);



