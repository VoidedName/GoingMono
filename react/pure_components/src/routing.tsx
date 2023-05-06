import {Link, NavLink, Outlet} from "react-router-dom";

export default function Navigation({title, links}: { title: string, links: { path: string, display: string, always_active?: boolean }[] }) {
    return <div className={"routing"}>
        <div className={"sidebar"}>
            <h1>{title}</h1>
            <nav>
                <ul>
                    {links.map(({path, display, always_active}) =>
                        <li key={path}>
                            <NavLink
                                to={path}
                                end={!always_active}
                            >
                                {display}
                            </NavLink>
                        </li>)}
                </ul>
            </nav>
        </div>
        <div className={"content"}>
            <Outlet/>
        </div>
    </div>
}
