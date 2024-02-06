import {
    Route,
    Navigate,
    createBrowserRouter,
    createRoutesFromElements,
    RouterProvider,
    Link,
} from 'react-router-dom';
import GameLayout, { gameLoader } from './components/GameLayout/GameLayout.tsx';

const router = createBrowserRouter(
    createRoutesFromElements(
        <>
            <Route
                path="/"
                element={
                    <div>
                        TODO Home
                        <Link to="/game/c793dbe7-5829-428a-a249-55a03eb091c9">
                            NAVIGATE TO GAME
                        </Link>
                    </div>
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
