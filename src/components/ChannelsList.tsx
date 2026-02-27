// import { useRef, useState } from 'react'
import { Channel } from '../types'

interface ChannelsListProps {
    channels: Channel[];
    setChannel: (channel: string) => void;
    channel: string;
}

function ChannelsList({ channels, setChannel, channel }: ChannelsListProps) {
  return (
        <ul>
            {channels.map((channelItem) => (
                <li key={channelItem.uuid} onClick={ () => { setChannel(channelItem.uuid) }} className='text-2xl flex flex-row px-3 py-2 text-neutral-400 hover:bg-neutral-800 focus:bg-neutral-800 rounded-xl' style={channel == channelItem.uuid ? { backgroundColor: "#1a1a1a" } : {}}>#<div className='w-full min-w-0 overflow-hidden mx-2 text-xl'>{channelItem.topic}</div> <div className='text-xl'>{ channelItem.unreadCount > 0 ? channelItem.unreadCount : '' }</div></li>
            ))}
        </ul>
    )
}

export default ChannelsList