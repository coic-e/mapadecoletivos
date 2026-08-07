import { createBrowserRouter, Navigate } from "react-router-dom";
import { Landing, Organization, RootError, OrganizationsMap, CreateOrganization } from "../pages";

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
    element: <OrganizationsMap />,
    errorElement: <RootError />,
  },
  {
    path: "/raves/create",
    element: <CreateOrganization />,
    errorElement: <RootError />,
  },
  {
    path: "/raves/:id",
    element: <Organization />,
    errorElement: <RootError />,
  },
  {
    path: "*",
    element: <Navigate to="/" />,
  },
]);
