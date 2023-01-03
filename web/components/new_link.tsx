import { useState } from 'react'
import { InputGroup, Input, InputRightElement, Button, InputLeftAddon } from '@chakra-ui/react'

export default function CreateLink() {
    const [url, setUrl] = useState('')

    const sendIt = async (url: String) => {
        const requestOptions = {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ "value": url })
        };

        const response = await fetch(`${import.meta.env.VITE_ENDPOINT}n`, requestOptions);
        const data = await response.json();
        console.log(data);
    }

    return (
        <>
            <InputGroup size='md'>
                <InputLeftAddon children='https://' />
                <Input
                    pr='4.5rem'
                    placeholder='your url'
                    onChange={e => setUrl(e.target.value)}
                />
                <InputRightElement width='6.45rem'>
                    <Button h='1.75rem' size='sm' color='black' bg='teal.100' onClick={() => sendIt(url)}>
                        Get Short
                    </Button>
                </InputRightElement>
            </InputGroup>
        </>
    )
}

