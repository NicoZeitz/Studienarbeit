import {
    Route,
    Navigate,
    createBrowserRouter,
    createRoutesFromElements,
    RouterProvider,
} from 'react-router-dom';
import GameLayout, { gameLoader } from './components/GameLayout/GameLayout.tsx';
import MainPage from './components/MainPage/MainPage.tsx';

const router = createBrowserRouter(
    createRoutesFromElements(
        <>
            <Route
                path="/"
                element={
                    <>
                        <MainPage />
                    </>
                }
            />

            <Route
                path="/game/:id"
                loader={gameLoader}
                element={<GameLayout />}
            />
            <Route path="*" element={<Navigate to="/" />} />
        </>,
    ),
);

export default function App() {
    return <RouterProvider router={router} />;
}
