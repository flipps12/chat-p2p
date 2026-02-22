import { MyInfo, Peer } from '../types';

interface UserListProps {
    peers?: Peer[];
    myInfo: MyInfo | null;
}

interface UserItemProps {
    name: string;
    peer: string;
}

function UserList({ myInfo, peers }: UserListProps) {
    let myPeer = myInfo?.peer_id;
    return (
    <ul>
        <UserItem name={"Flipps"} peer={myPeer || 'No peer detected'} />
        {peers?.map((peer) => (
            <UserItem key={peer.peer_id} name={"Unknown"} peer={peer?.peer_id || 'No peer detected'} />
        ))}
    </ul>
    );
}

function UserItem({ name, peer }: UserItemProps) {
    return (
        <li onClick={ () => {  }} className='text-2xl flex flex-col px-3 py-2 text-neutral-400 hover:bg-neutral-800 focus:bg-neutral-800 rounded-xl'>
        <div className='w-full min-w-0 overflow-hidden mx-2 text-xl'>{name}</div>
        <div className='w-full min-w-0 overflow-hidden mx-2 text-xs'>{peer}</div>
        </li>
    )
}

export default UserList